use std::path::Path;
use std::sync::Arc;

use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use tokio::sync::mpsc;

use crate::domain::ports::Synchronisation;

/// Watches `files_dir` for file write and create events. When a write is
/// detected, calls [`Synchronisation::synchronise`] via a single dedicated
/// blocking task, so concurrent file-save bursts are collapsed into one sync
/// operation rather than spawning unbounded parallel tasks.
///
/// # Errors
///
/// Returns an error if the watcher cannot be initialised or if the target
/// directory cannot be watched.
pub async fn watch(files_dir: &str, sync: Arc<dyn Synchronisation>) -> NotifyResult<()> {
    // Channel from the OS watcher thread into the async runtime.
    let (event_tx, mut event_rx) = mpsc::channel::<NotifyResult<Event>>(64);
    let rt = tokio::runtime::Handle::current();

    let mut watcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            let event_tx = event_tx.clone();
            rt.spawn(async move {
                let _ = event_tx.send(res).await;
            });
        },
        Config::default(),
    )?;

    watcher.watch(Path::new(files_dir), RecursiveMode::Recursive)?;

    println!("Daemon watching directory: {}", files_dir);

    // Single-consumer sync channel: capacity 1 acts as a "pending" flag.
    // If a sync is already queued, additional events are dropped — they will
    // be picked up by the next sync run anyway (git stages all dirty files).
    let (sync_tx, mut sync_rx) = mpsc::channel::<()>(1);

    // Dedicated blocking task for git operations.
    let sync_clone = Arc::clone(&sync);
    tokio::spawn(async move {
        while sync_rx.recv().await.is_some() {
            // Drain any additional queued signals so we don't double-sync.
            while sync_rx.try_recv().is_ok() {}

            let sync = Arc::clone(&sync_clone);
            let result = tokio::task::spawn_blocking(move || sync.synchronise()).await;

            match result {
                Ok(Ok(report)) => println!("Sync complete: commit {}", report.commit_name),
                Ok(Err(e)) => eprintln!("Sync error: {:?}", e),
                Err(e) => eprintln!("Sync task panicked: {:?}", e),
            }
        }
    });

    while let Some(res) = event_rx.recv().await {
        match res {
            Ok(event) => {
                let is_write = matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(_)
                );

                if is_write {
                    // Non-blocking send: if the channel is full a sync is
                    // already pending, so this event is intentionally dropped.
                    let _ = sync_tx.try_send(());
                }
            }
            Err(e) => eprintln!("Watcher error: {:?}", e),
        }
    }

    Ok(())
}
