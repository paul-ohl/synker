pub trait Synchronisation {
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError>;
    fn is_synchronised(&self) -> Result<SynchronisationReport, SynchronisationError>;
}

pub struct SynchronisationReport {
    pub commit_name: String,
    pub last_sync_time: Option<std::time::SystemTime>,
    pub last_sync_duration: Option<std::time::Duration>,
    pub pending_changes: usize,
}

pub enum SynchronisationError {
    NetworkError(String),
    SyncToolError(String),
    FileConflict(String),
    UnknownError(String),
}
