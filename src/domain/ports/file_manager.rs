use uuid::Uuid;

use crate::domain::services::file_manager::FileManagerError;
use crate::domain::types::file::{
    file::{File, NewFile, UpdateFile},
    file_metadata::Metadata,
    file_query::FileQuery,
};

/// Outbound port: defines the raw storage operations that an infrastructure
/// adapter must provide. Implemented by adapters in the `outbound` layer
/// (e.g., `FsFileManager`).
pub trait FileManager: Send + Sync {
    fn create_file(&self, new_file: NewFile) -> Result<File, FileManagerError>;
    fn create_file_bytes(
        &self,
        name: String,
        ext: String,
        mime: String,
        tags: Vec<String>,
        data: Vec<u8>,
    ) -> Result<Metadata, FileManagerError>;
    fn read_file(&self, file_id: Uuid) -> Result<File, FileManagerError>;
    fn read_file_bytes(&self, file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError>;
    fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError>;
    fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError>;
    fn update_file(&self, file_id: Uuid, update: UpdateFile) -> Result<File, FileManagerError>;
    fn delete_file(&self, file_id: Uuid) -> Result<(), FileManagerError>;
    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError>;
}
