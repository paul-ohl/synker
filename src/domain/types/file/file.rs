use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::types::file::file_metadata::{
    Metadata, MetadataCreationError, NewMetadata, UpdateMetadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub metadata: Metadata,
    pub content: String,
}

pub struct NewFile {
    pub metadata: NewMetadata,
    pub content: Option<String>,
}

impl NewFile {
    pub fn new(
        metadata: NewMetadata,
        content: Option<String>,
    ) -> Result<Self, MetadataCreationError> {
        let new_metadata =
            NewMetadata::new(metadata.name, metadata.ext, metadata.tags, metadata.mime)?;
        let new_file = Self {
            metadata: new_metadata,
            content,
        };
        Ok(new_file)
    }
}

pub struct UpdateFile {
    pub metadata: Option<UpdateMetadata>,
    pub content: Option<String>,
}

impl UpdateFile {
    pub fn new(
        metadata: Option<UpdateMetadata>,
        content: Option<String>,
    ) -> Result<Self, MetadataCreationError> {
        let update_metadata = match metadata {
            Some(meta) => Some(UpdateMetadata::new(
                meta.name, meta.ext, meta.tags, meta.mime,
            )?),
            None => None,
        };
        let update_file = Self {
            metadata: update_metadata,
            content,
        };
        Ok(update_file)
    }
}
