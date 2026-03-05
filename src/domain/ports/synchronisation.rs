use crate::domain::services::synchronisation::{SynchronisationError, SynchronisationReport};

pub trait Synchronisation: Send + Sync {
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError>;
    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError>;
}
