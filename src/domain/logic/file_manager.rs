use std::sync::Arc;

use uuid::Uuid;

use crate::domain::types::file::{
    file::{File, NewFile, UpdateFile},
    file_metadata::Metadata,
    file_query::FileQuery,
};
use crate::domain::{
    ports,
    services::file_manager::{self, FileManagerError},
};

/// Domain logic orchestrator for file management.
/// Implements the `services::FileManager` trait, applying business rules
/// before delegating raw storage operations to a `ports::FileManager` adapter.
pub struct FileManagerLogic {
    adapter: Arc<dyn ports::FileManager>,
}

impl FileManagerLogic {
    pub fn new(adapter: Arc<dyn ports::FileManager>) -> Self {
        Self { adapter }
    }
}

impl file_manager::FileManager for FileManagerLogic {
    fn create_file(&self, new_file: NewFile) -> Result<File, FileManagerError> {
        // Business rule: validate the NewFile metadata
        let validated = NewFile::new(new_file.metadata, new_file.content)
            .map_err(|e| FileManagerError::ValidationError(e.to_string()))?;

        self.adapter.create_file(validated)
    }

    fn create_file_bytes(
        &self,
        name: String,
        ext: String,
        mime: String,
        tags: Vec<String>,
        data: Vec<u8>,
    ) -> Result<Metadata, FileManagerError> {
        if name.trim().is_empty() {
            return Err(FileManagerError::ValidationError(
                "File name cannot be empty".to_string(),
            ));
        }
        if ext.trim().is_empty() {
            return Err(FileManagerError::ValidationError(
                "Extension cannot be empty".to_string(),
            ));
        }
        self.adapter.create_file_bytes(name, ext, mime, tags, data)
    }

    fn read_file(&self, file_id: Uuid) -> Result<File, FileManagerError> {
        self.adapter.read_file(file_id)
    }

    fn read_file_bytes(&self, file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError> {
        self.adapter.read_file_bytes(file_id)
    }

    fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError> {
        self.adapter.list_files()
    }

    fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError> {
        self.adapter.list_all_tags()
    }

    fn update_file(&self, file_id: Uuid, update: UpdateFile) -> Result<File, FileManagerError> {
        // Validate update before delegating
        let validated = UpdateFile::new(update.metadata, update.content)
            .map_err(|e| FileManagerError::ValidationError(e.to_string()))?;

        self.adapter.update_file(file_id, validated)
    }

    fn delete_file(&self, file_id: Uuid) -> Result<(), FileManagerError> {
        self.adapter.delete_file(file_id)
    }

    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError> {
        query.verify().map_err(FileManagerError::ValidationError)?;

        self.adapter.find(query)
    }
}
