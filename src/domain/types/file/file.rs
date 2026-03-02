use uuid::Uuid;

use crate::domain::types::file::file_metadata::{Metadata, NewMetadata, UpdateMetadata};

pub struct File {
    pub id: Uuid,
    pub metadata: Metadata,
    pub content: String,
}

pub struct NewFile {
    pub metadata: NewMetadata,
    pub content: Option<String>,
}

pub struct UpdateFile {
    pub metadata: Option<UpdateMetadata>,
    pub content: Option<String>,
}
