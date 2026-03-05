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
        if let Some(name) = &self.name
            && name.trim().is_empty()
        {
            return Err(MetadataCreationError::InvalidName(name.clone()));
        }
        if let Some(ext) = &self.ext
            && ext.trim().is_empty()
        {
            return Err(MetadataCreationError::InvalidExtension(ext.clone()));
        }
        if let Some(tags) = &self.tags
            && tags.iter().any(|tag| tag.trim().is_empty())
        {
            return Err(MetadataCreationError::InvalidTags(format!(
                "Tags cannot contain empty strings: {:?}",
                tags
            )));
        }
        if let Some(mime) = &self.mime
            && mime.trim().is_empty()
        {
            return Err(MetadataCreationError::InvalidMimeType(mime.clone()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NewMetadata ──────────────────────────────────────────────────────────

    #[test]
    fn test_new_metadata_valid() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "md".to_string(),
            vec!["docs".to_string(), "rust".to_string()],
            "text/markdown".to_string(),
        );
        assert!(result.is_ok());
        let m = result.unwrap();
        assert_eq!(m.name, "readme");
        assert_eq!(m.ext, "md");
        assert_eq!(m.tags, vec!["docs", "rust"]);
        assert_eq!(m.mime, "text/markdown");
    }

    #[test]
    fn test_new_metadata_empty_name() {
        let result = NewMetadata::new(
            "".to_string(),
            "md".to_string(),
            vec![],
            "text/plain".to_string(),
        );
        assert!(matches!(result, Err(MetadataCreationError::InvalidName(_))));
    }

    #[test]
    fn test_new_metadata_whitespace_name() {
        let result = NewMetadata::new(
            "   ".to_string(),
            "md".to_string(),
            vec![],
            "text/plain".to_string(),
        );
        assert!(matches!(result, Err(MetadataCreationError::InvalidName(_))));
    }

    #[test]
    fn test_new_metadata_empty_ext() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "".to_string(),
            vec![],
            "text/plain".to_string(),
        );
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidExtension(_))
        ));
    }

    #[test]
    fn test_new_metadata_whitespace_ext() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "  ".to_string(),
            vec![],
            "text/plain".to_string(),
        );
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidExtension(_))
        ));
    }

    #[test]
    fn test_new_metadata_empty_tag_in_list() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "md".to_string(),
            vec!["good-tag".to_string(), "".to_string()],
            "text/plain".to_string(),
        );
        assert!(matches!(result, Err(MetadataCreationError::InvalidTags(_))));
    }

    #[test]
    fn test_new_metadata_whitespace_tag_in_list() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "md".to_string(),
            vec!["   ".to_string()],
            "text/plain".to_string(),
        );
        assert!(matches!(result, Err(MetadataCreationError::InvalidTags(_))));
    }

    #[test]
    fn test_new_metadata_empty_tags_list_is_ok() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "md".to_string(),
            vec![],
            "text/plain".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_metadata_empty_mime() {
        let result = NewMetadata::new(
            "readme".to_string(),
            "md".to_string(),
            vec![],
            "".to_string(),
        );
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidMimeType(_))
        ));
    }

    #[test]
    fn test_new_metadata_display_errors() {
        assert_eq!(
            MetadataCreationError::InvalidName("".to_string()).to_string(),
            "Invalid name: ''"
        );
        assert_eq!(
            MetadataCreationError::InvalidExtension("".to_string()).to_string(),
            "Invalid extension: ''"
        );
        assert_eq!(
            MetadataCreationError::InvalidTags("bad tags".to_string()).to_string(),
            "Invalid tags: bad tags"
        );
        assert_eq!(
            MetadataCreationError::InvalidMimeType("".to_string()).to_string(),
            "Invalid MIME type: ''"
        );
        assert_eq!(
            MetadataCreationError::InvalidSize(0).to_string(),
            "Invalid size: 0"
        );
        assert_eq!(
            MetadataCreationError::InvalidTimestamps(
                "2020-01-01".to_string(),
                "2019-01-01".to_string()
            )
            .to_string(),
            "Invalid timestamps: 2020-01-01, 2019-01-01"
        );
    }

    // ── UpdateMetadata ───────────────────────────────────────────────────────

    #[test]
    fn test_update_metadata_valid_all_none() {
        let result = UpdateMetadata::new(None, None, None, None);
        assert!(result.is_ok());
        let m = result.unwrap();
        assert!(m.name.is_none());
        assert!(m.ext.is_none());
        assert!(m.tags.is_none());
        assert!(m.mime.is_none());
    }

    #[test]
    fn test_update_metadata_valid_some_fields() {
        let result = UpdateMetadata::new(
            Some("new-name".to_string()),
            Some("txt".to_string()),
            Some(vec!["tag1".to_string()]),
            Some("text/plain".to_string()),
        );
        assert!(result.is_ok());
        let m = result.unwrap();
        assert_eq!(m.name.as_deref(), Some("new-name"));
        assert_eq!(m.ext.as_deref(), Some("txt"));
        assert_eq!(m.mime.as_deref(), Some("text/plain"));
    }

    #[test]
    fn test_update_metadata_empty_name() {
        let result = UpdateMetadata::new(Some("".to_string()), None, None, None);
        assert!(matches!(result, Err(MetadataCreationError::InvalidName(_))));
    }

    #[test]
    fn test_update_metadata_empty_ext() {
        let result = UpdateMetadata::new(None, Some("".to_string()), None, None);
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidExtension(_))
        ));
    }

    #[test]
    fn test_update_metadata_empty_tag() {
        let result = UpdateMetadata::new(
            None,
            None,
            Some(vec!["ok".to_string(), "".to_string()]),
            None,
        );
        assert!(matches!(result, Err(MetadataCreationError::InvalidTags(_))));
    }

    #[test]
    fn test_update_metadata_empty_mime() {
        let result = UpdateMetadata::new(None, None, None, Some("".to_string()));
        assert!(matches!(
            result,
            Err(MetadataCreationError::InvalidMimeType(_))
        ));
    }
}
