use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::domain::{ports::Synchronisation, services::synchronisation::SynchronisationError};

pub async fn watch(sync: Arc<dyn Synchronisation>, sync_delay_hours: u64) {
    info!(sync_delay_hours, "Daemon timer started");

    loop {
        let report = sync.get_last_sync();

        let needs_sync = match report {
            Ok(r) => {
                let time_threshold_exceeded = match SystemTime::now().duration_since(r.last_sync_time) {
                    Ok(duration) => duration.as_secs() > sync_delay_hours * 3600,
                    Err(_) => {
                        warn!("Clock skew detected; assuming sync is needed");
                        true
                    }
                };

                // Sync if we have pending changes OR it's been too long since last sync
                let result = r.pending_changes > 0 || time_threshold_exceeded;
                if result {
                    info!(
                        pending_changes = r.pending_changes,
                        time_threshold_exceeded,
                        "Sync condition met"
                    );
                }
                result
            }
            Err(SynchronisationError::FirstTimeSync) => {
                info!("First-time sync detected; proceeding to sync");
                true
            }
            Err(e) => {
                error!(error = ?e, "Error checking sync status; skipping this cycle");
                false
            }
        };

        if needs_sync {
            info!(sync_delay_hours, "Starting sync attempt");
            loop {
                match sync.synchronise() {
                    Ok(report) => {
                        info!(commit = %report.commit_name, "Sync complete");
                        break;
                    }
                    Err(e) => {
                        error!(error = ?e, "Sync failed; retrying in 2 minutes");
                        sleep(Duration::from_secs(120)).await;
                    }
                }
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}
