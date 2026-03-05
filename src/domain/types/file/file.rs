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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_new_metadata() -> NewMetadata {
        NewMetadata {
            name: "my-file".to_string(),
            ext: "md".to_string(),
            tags: vec!["docs".to_string()],
            mime: "text/markdown".to_string(),
        }
    }

    #[test]
    fn test_new_file_valid() {
        let result = NewFile::new(valid_new_metadata(), Some("Hello world".to_string()));
        assert!(result.is_ok());
        let f = result.unwrap();
        assert_eq!(f.metadata.name, "my-file");
        assert_eq!(f.metadata.ext, "md");
        assert_eq!(f.content, Some("Hello world".to_string()));
    }

    #[test]
    fn test_new_file_invalid_metadata() {
        // Empty name should cause validation failure
        let bad_meta = NewMetadata {
            name: "".to_string(),
            ext: "md".to_string(),
            tags: vec![],
            mime: "text/plain".to_string(),
        };
        let result = NewFile::new(bad_meta, None);
        assert!(matches!(result, Err(MetadataCreationError::InvalidName(_))));
    }

    #[test]
    fn test_update_file_valid_no_metadata() {
        let result = UpdateFile::new(None, Some("updated content".to_string()));
        assert!(result.is_ok());
        let f = result.unwrap();
        assert!(f.metadata.is_none());
        assert_eq!(f.content, Some("updated content".to_string()));
    }

    #[test]
    fn test_update_file_valid_with_metadata() {
        let meta = UpdateMetadata {
            name: Some("new-name".to_string()),
            ext: Some("txt".to_string()),
            tags: None,
            mime: None,
        };
        let result = UpdateFile::new(Some(meta), None);
        assert!(result.is_ok());
        let f = result.unwrap();
        assert!(f.metadata.is_some());
        let m = f.metadata.unwrap();
        assert_eq!(m.name.as_deref(), Some("new-name"));
        assert_eq!(m.ext.as_deref(), Some("txt"));
    }

    #[test]
    fn test_update_file_invalid_metadata() {
        // Empty extension in update metadata should propagate error
        let bad_meta = UpdateMetadata {
            name: None,
            ext: Some("".to_string()),
            tags: None,
            mime: None,
        };
        let result = UpdateFile::new(Some(bad_meta), None);
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidExtension(_))
        ));
    }
}
