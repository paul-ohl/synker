use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::time::sleep;

use crate::domain::{ports::Synchronisation, services::synchronisation::SynchronisationError};

pub async fn watch(sync: Arc<dyn Synchronisation>, sync_delay_hours: u64) {
    println!(
        "Daemon timer started with SYNC_DELAY_HOURS: {} hours",
        sync_delay_hours
    );

    loop {
        let report = sync.get_last_sync();

        let needs_sync = match report {
            Ok(r) => {
                let time_threshold_exceeded = match SystemTime::now().duration_since(r.last_sync_time) {
                    Ok(duration) => duration.as_secs() > sync_delay_hours * 3600,
                    Err(_) => true, // Clock skew, assume sync needed
                };

                // Sync if we have pending changes OR it's been too long since last sync
                r.pending_changes > 0 || time_threshold_exceeded
            }
            Err(SynchronisationError::FirstTimeSync) => {
                println!("First time sync detected. Proceeding to sync.");
                true
            }
            Err(e) => {
                eprintln!("Error checking sync status: {:?}", e);
                // If we can't check status, we probably shouldn't blindly sync,
                // but maybe we should retry checking soon.
                false
            }
        };

        if needs_sync {
            println!(
                "Sync needed (last sync > {}h ago). Starting sync attempts...",
                sync_delay_hours
            );
            loop {
                match sync.synchronise() {
                    Ok(report) => {
                        println!("Sync complete: commit {}", report.commit_name);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Sync error: {:?}. Retrying in 2 minutes...", e);
                        sleep(Duration::from_secs(120)).await;
                    }
                }
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}
