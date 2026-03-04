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
