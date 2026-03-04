pub trait Synchronisation {
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError>;
    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError>;
}

pub struct SynchronisationReport {
    pub commit_name: String,
    pub last_sync_time: Option<std::time::SystemTime>,
    pub last_sync_duration: Option<std::time::Duration>,
    pub pending_changes: usize,
}

#[derive(Debug)]
pub enum SynchronisationError {
    NetworkError(String),
    SyncToolError(String),
    FileConflict(String),
    UnknownError(String),
}
