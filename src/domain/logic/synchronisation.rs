use crate::domain::{
    ports,
    services::synchronisation::{self, SynchronisationError, SynchronisationReport},
};

pub struct Synchronisation {
    adapter: Box<dyn ports::Synchronisation>,
}

impl Synchronisation {
    pub fn new(adapter: Box<dyn ports::Synchronisation>) -> Self {
        Self { adapter }
    }
}

impl synchronisation::Synchronisation for Synchronisation {
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError> {
        if self.adapter.register_changes()? {
            self.adapter.synchronise()
        } else {
            Ok(SynchronisationReport {
                commit_name: "No changes to synchronise".to_string(),
                last_sync_time: None,
                last_sync_duration: None,
                pending_changes: 0,
            })
        }
    }

    fn is_synchronised(&self) -> Result<SynchronisationReport, SynchronisationError> {
        self.adapter.is_synchronised()
    }
}
