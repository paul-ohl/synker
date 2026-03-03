use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: Uuid,
    pub name: String,
    pub ext: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub mime: String,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug)]
pub enum MetadataCreationError {
    InvalidName(String),
    InvalidExtension(String),
    InvalidTags(String),
    InvalidMimeType(String),
    InvalidSize(u64),
    InvalidTimestamps(String, String),
}

impl fmt::Display for MetadataCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(n) => write!(f, "Invalid name: '{n}'"),
            Self::InvalidExtension(e) => write!(f, "Invalid extension: '{e}'"),
            Self::InvalidTags(t) => write!(f, "Invalid tags: {t}"),
            Self::InvalidMimeType(m) => write!(f, "Invalid MIME type: '{m}'"),
            Self::InvalidSize(s) => write!(f, "Invalid size: {s}"),
            Self::InvalidTimestamps(a, b) => write!(f, "Invalid timestamps: {a}, {b}"),
        }
    }
}

impl std::error::Error for MetadataCreationError {}

pub struct NewMetadata {
    pub name: String,
    pub ext: String,
    pub tags: Vec<String>,
    pub mime: String,
}

impl NewMetadata {
    pub fn new(
        name: String,
        ext: String,
        tags: Vec<String>,
        mime: String,
    ) -> Result<Self, MetadataCreationError> {
        let new_metadata = Self {
            name,
            ext,
            tags,
            mime,
        };
        new_metadata.validate()?;
        Ok(new_metadata)
    }

    fn validate(&self) -> Result<(), MetadataCreationError> {
        if self.name.trim().is_empty() {
            return Err(MetadataCreationError::InvalidName(self.name.clone()));
        }
        if self.ext.trim().is_empty() {
            return Err(MetadataCreationError::InvalidExtension(self.ext.clone()));
        }
        if self.tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(MetadataCreationError::InvalidTags(format!(
                "Tags cannot contain empty strings: {:?}",
                self.tags
            )));
        }
        if self.mime.trim().is_empty() {
            return Err(MetadataCreationError::InvalidMimeType(self.mime.clone()));
        }
        Ok(())
    }
}

pub struct UpdateMetadata {
    pub name: Option<String>,
    pub ext: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime: Option<String>,
}

impl UpdateMetadata {
    pub fn new(
        name: Option<String>,
        ext: Option<String>,
        tags: Option<Vec<String>>,
        mime: Option<String>,
    ) -> Result<Self, MetadataCreationError> {
        let update_metadata = Self {
            name,
            ext,
            tags,
            mime,
        };
        update_metadata.validate()?;
        Ok(update_metadata)
    }

    fn validate(&self) -> Result<(), MetadataCreationError> {
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err(MetadataCreationError::InvalidName(name.clone()));
            }
        }
        if let Some(ext) = &self.ext {
            if ext.trim().is_empty() {
                return Err(MetadataCreationError::InvalidExtension(ext.clone()));
            }
        }
        if let Some(tags) = &self.tags {
            if tags.iter().any(|tag| tag.trim().is_empty()) {
                return Err(MetadataCreationError::InvalidTags(format!(
                    "Tags cannot contain empty strings: {:?}",
                    tags
                )));
            }
        }
        if let Some(mime) = &self.mime {
            if mime.trim().is_empty() {
                return Err(MetadataCreationError::InvalidMimeType(mime.clone()));
            }
        }
        Ok(())
    }
}
