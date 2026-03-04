use crate::domain::services::synchronisation::{SynchronisationError, SynchronisationReport};

pub trait Synchronisation: Send + Sync {
    /// Registers any changes that need to be synchronised across the system.
    /// Returns `Ok(true)` if changes were registered successfully, `Ok(false)` if there were no changes to register, or an error if the registration failed.
    fn register_changes(&self) -> Result<bool, SynchronisationError>;

    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError>;
    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError>;
}
