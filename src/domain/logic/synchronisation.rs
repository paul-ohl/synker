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
        self.adapter.synchronise()
    }

    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError> {
        self.adapter.get_last_sync()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports, services::synchronisation::Synchronisation as SynchronisationTrait,
    };

    struct MockSync {
        synchronise_result: Result<SynchronisationReport, SynchronisationError>,
        get_last_sync_result: Result<SynchronisationReport, SynchronisationError>,
    }

    fn ok_report(commit: &str) -> SynchronisationReport {
        SynchronisationReport {
            commit_name: commit.to_string(),
            last_sync_time: std::time::SystemTime::UNIX_EPOCH,
            pending_changes: 0,
        }
    }

    impl ports::Synchronisation for MockSync {
        fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError> {
            match &self.synchronise_result {
                Ok(r) => Ok(SynchronisationReport {
                    commit_name: r.commit_name.clone(),
                    last_sync_time: r.last_sync_time,
                    pending_changes: r.pending_changes,
                }),
                Err(SynchronisationError::FirstTimeSync) => {
                    Err(SynchronisationError::FirstTimeSync)
                }
                Err(SynchronisationError::NetworkError(s)) => {
                    Err(SynchronisationError::NetworkError(s.clone()))
                }
                Err(SynchronisationError::SyncToolError(s)) => {
                    Err(SynchronisationError::SyncToolError(s.clone()))
                }
                Err(SynchronisationError::FileConflict(s)) => {
                    Err(SynchronisationError::FileConflict(s.clone()))
                }
                Err(SynchronisationError::UnknownError(s)) => {
                    Err(SynchronisationError::UnknownError(s.clone()))
                }
            }
        }

        fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError> {
            match &self.get_last_sync_result {
                Ok(r) => Ok(SynchronisationReport {
                    commit_name: r.commit_name.clone(),
                    last_sync_time: r.last_sync_time,
                    pending_changes: r.pending_changes,
                }),
                Err(SynchronisationError::FirstTimeSync) => {
                    Err(SynchronisationError::FirstTimeSync)
                }
                Err(SynchronisationError::NetworkError(s)) => {
                    Err(SynchronisationError::NetworkError(s.clone()))
                }
                Err(SynchronisationError::SyncToolError(s)) => {
                    Err(SynchronisationError::SyncToolError(s.clone()))
                }
                Err(SynchronisationError::FileConflict(s)) => {
                    Err(SynchronisationError::FileConflict(s.clone()))
                }
                Err(SynchronisationError::UnknownError(s)) => {
                    Err(SynchronisationError::UnknownError(s.clone()))
                }
            }
        }
    }

    #[test]
    fn test_synchronise_delegates_to_adapter() {
        let mock = MockSync {
            synchronise_result: Ok(ok_report("abc123")),
            get_last_sync_result: Err(SynchronisationError::FirstTimeSync),
        };
        let logic = Synchronisation::new(Box::new(mock));

        let result = logic.synchronise();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().commit_name, "abc123");
    }

    #[test]
    fn test_get_last_sync_delegates_to_adapter() {
        let mock = MockSync {
            synchronise_result: Err(SynchronisationError::FirstTimeSync),
            get_last_sync_result: Ok(ok_report("def456")),
        };
        let logic = Synchronisation::new(Box::new(mock));

        let result = logic.get_last_sync();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().commit_name, "def456");
    }

    #[test]
    fn test_synchronise_propagates_error() {
        let mock = MockSync {
            synchronise_result: Err(SynchronisationError::NetworkError("timeout".to_string())),
            get_last_sync_result: Err(SynchronisationError::FirstTimeSync),
        };
        let logic = Synchronisation::new(Box::new(mock));

        let result = logic.synchronise();
        assert!(matches!(result, Err(SynchronisationError::NetworkError(_))));
    }

    #[test]
    fn test_get_last_sync_propagates_first_time_sync() {
        let mock = MockSync {
            synchronise_result: Err(SynchronisationError::FirstTimeSync),
            get_last_sync_result: Err(SynchronisationError::FirstTimeSync),
        };
        let logic = Synchronisation::new(Box::new(mock));

        let result = logic.get_last_sync();
        assert!(matches!(result, Err(SynchronisationError::FirstTimeSync)));
    }
}
