pub trait Synchronisation {
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError>;
    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError>;
}

#[derive(Debug)]
pub struct SynchronisationReport {
    pub commit_name: String,
    pub last_sync_time: std::time::SystemTime,
    pub pending_changes: usize,
}

#[derive(Debug)]
pub enum SynchronisationError {
    FirstTimeSync,
    NetworkError(String),
    SyncToolError(String),
    FileConflict(String),
    UnknownError(String),
}
