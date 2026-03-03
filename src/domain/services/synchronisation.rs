pub trait Synchronisation {
    fn synchronise(&self) -> SynchronisationReport;
    fn is_synchronised(&self) -> SynchronisationReport;
}

pub struct SynchronisationReport {
    pub success: bool,
    pub commit_name: String,
    pub last_sync_time: Option<std::time::SystemTime>,
    pub last_sync_duration: Option<std::time::Duration>,
    pub pending_changes: usize,
    pub error: Option<SynchronisationError>,
}

pub enum SynchronisationError {
    NetworkError(String),
    AuthenticationError(String),
    FileConflict(String),
    UnknownError(String),
}
