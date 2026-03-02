use uuid::Uuid;

use crate::domain::types::file::{
    file::{File, NewFile, UpdateFile},
    file_metadata::Metadata,
    file_query::FileQuery,
};

pub trait FileManager {
    fn create_files(&self, new_file: Vec<NewFile>) -> Result<Vec<File>, FileManagerError>;
    fn read_files(&self, file_ids: Vec<Uuid>) -> Result<Vec<File>, FileManagerError>;
    fn update_files(&self, files: Vec<UpdateFile>) -> Result<Vec<File>, FileManagerError>;
    fn delete_files(&self, file_ids: Vec<Uuid>) -> Result<(), FileManagerError>;

    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError>;
}

pub enum FileManagerError {
    FileNotFound(String),
    FileAlreadyExists(String),
    PermissionDenied(String),
    UnknownError(String),
}
