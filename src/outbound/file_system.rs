use std::fs;
use std::io;
use std::path::PathBuf;

use uuid::Uuid;

use crate::domain::services::file_manager::{FileManager, FileManagerError};
use crate::domain::types::file::{
    file::{File, NewFile, UpdateFile},
    file_metadata::Metadata,
    file_query::FileQuery,
};

/// Outbound adapter: filesystem-based implementation of the FileManager port.
///
/// Files are stored as:
///   `{base_path}/{uuid}.{ext}`
///
/// Metadata is stored alongside as:
///   `{base_path}/.meta/{uuid}.json`
pub struct FsFileManager {
    base_path: PathBuf,
    meta_path: PathBuf,
}

impl FsFileManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Result<Self, io::Error> {
        let base_path = base_path.into();
        let meta_path = base_path.join(".meta");

        fs::create_dir_all(&base_path)?;
        fs::create_dir_all(&meta_path)?;

        Ok(Self {
            base_path,
            meta_path,
        })
    }

    fn file_path(&self, id: Uuid, ext: &str) -> PathBuf {
        self.base_path.join(format!("{}.{}", id, ext))
    }

    fn meta_file_path(&self, id: Uuid) -> PathBuf {
        self.meta_path.join(format!("{}.json", id))
    }

    fn write_metadata(&self, metadata: &Metadata) -> Result<(), FileManagerError> {
        let json = serde_json::to_string_pretty(metadata)
            .map_err(|e| FileManagerError::IoError(e.to_string()))?;
        fs::write(self.meta_file_path(metadata.id), json)
            .map_err(|e| FileManagerError::IoError(e.to_string()))?;
        Ok(())
    }

    fn read_metadata(&self, id: Uuid) -> Result<Metadata, FileManagerError> {
        let path = self.meta_file_path(id);
        let json = fs::read_to_string(&path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                FileManagerError::FileNotFound(format!("Metadata not found for {id}"))
            }
            _ => FileManagerError::IoError(e.to_string()),
        })?;
        let metadata: Metadata =
            serde_json::from_str(&json).map_err(|e| FileManagerError::IoError(e.to_string()))?;
        Ok(metadata)
    }

    fn now_iso(&self) -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

impl FileManager for FsFileManager {
    fn create_file(&self, new_file: NewFile) -> Result<File, FileManagerError> {
        let id = Uuid::new_v4();
        let content = new_file.content.unwrap_or_default();
        let ext = &new_file.metadata.ext;
        let file_path = self.file_path(id, ext);

        // Write content to disk
        fs::write(&file_path, &content).map_err(|e| FileManagerError::IoError(e.to_string()))?;

        let size = content.len() as u64;
        let now = self.now_iso();

        let metadata = Metadata {
            id,
            name: new_file.metadata.name,
            ext: ext.clone(),
            tags: new_file.metadata.tags,
            size,
            mime: new_file.metadata.mime,
            created_at: now.clone(),
            modified_at: now,
        };

        self.write_metadata(&metadata)?;

        Ok(File {
            id,
            metadata,
            content,
        })
    }

    fn create_file_bytes(
        &self,
        name: String,
        ext: String,
        mime: String,
        tags: Vec<String>,
        data: Vec<u8>,
    ) -> Result<Metadata, FileManagerError> {
        let id = Uuid::new_v4();
        let file_path = self.file_path(id, &ext);

        fs::write(&file_path, &data).map_err(|e| FileManagerError::IoError(e.to_string()))?;

        let size = data.len() as u64;
        let now = self.now_iso();

        let metadata = Metadata {
            id,
            name,
            ext,
            tags,
            size,
            mime,
            created_at: now.clone(),
            modified_at: now,
        };

        self.write_metadata(&metadata)?;
        Ok(metadata)
    }

    fn read_file(&self, file_id: Uuid) -> Result<File, FileManagerError> {
        let metadata = self.read_metadata(file_id)?;
        let file_path = self.file_path(file_id, &metadata.ext);

        let content = fs::read_to_string(&file_path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                FileManagerError::FileNotFound(format!("File not found: {file_id}"))
            }
            _ => FileManagerError::IoError(e.to_string()),
        })?;

        Ok(File {
            id: file_id,
            metadata,
            content,
        })
    }

    fn read_file_bytes(&self, file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError> {
        let metadata = self.read_metadata(file_id)?;
        let file_path = self.file_path(file_id, &metadata.ext);

        let data = fs::read(&file_path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                FileManagerError::FileNotFound(format!("File not found: {file_id}"))
            }
            _ => FileManagerError::IoError(e.to_string()),
        })?;

        Ok((metadata, data))
    }

    fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError> {
        let mut files = Vec::new();

        let entries =
            fs::read_dir(&self.meta_path).map_err(|e| FileManagerError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| FileManagerError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "json") {
                let json = fs::read_to_string(&path)
                    .map_err(|e| FileManagerError::IoError(e.to_string()))?;
                if let Ok(metadata) = serde_json::from_str::<Metadata>(&json) {
                    files.push(metadata);
                }
            }
        }

        // Sort by modified_at descending (newest first)
        files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        Ok(files)
    }

    fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError> {
        let files = self.list_files()?;
        let mut tag_set = std::collections::BTreeSet::new();
        for f in &files {
            for t in &f.tags {
                tag_set.insert(t.clone());
            }
        }
        Ok(tag_set.into_iter().collect())
    }

    fn update_file(&self, file_id: Uuid, update: UpdateFile) -> Result<File, FileManagerError> {
        let mut metadata = self.read_metadata(file_id)?;
        let old_ext = metadata.ext.clone();

        // Apply metadata updates
        if let Some(meta_update) = update.metadata {
            if let Some(name) = meta_update.name {
                metadata.name = name;
            }
            if let Some(ext) = meta_update.ext {
                metadata.ext = ext;
            }
            if let Some(tags) = meta_update.tags {
                metadata.tags = tags;
            }
            if let Some(mime) = meta_update.mime {
                metadata.mime = mime;
            }
        }

        // Read existing content or use updated content
        let content = if let Some(new_content) = update.content {
            // Write new content
            let file_path = self.file_path(file_id, &metadata.ext);
            fs::write(&file_path, &new_content)
                .map_err(|e| FileManagerError::IoError(e.to_string()))?;

            // If extension changed, remove old file
            if old_ext != metadata.ext {
                let old_path = self.file_path(file_id, &old_ext);
                let _ = fs::remove_file(old_path);
            }

            metadata.size = new_content.len() as u64;
            new_content
        } else {
            // If extension changed, rename the file
            if old_ext != metadata.ext {
                let old_path = self.file_path(file_id, &old_ext);
                let new_path = self.file_path(file_id, &metadata.ext);
                fs::rename(&old_path, &new_path)
                    .map_err(|e| FileManagerError::IoError(e.to_string()))?;
            }
            let file_path = self.file_path(file_id, &metadata.ext);
            fs::read_to_string(&file_path).map_err(|e| FileManagerError::IoError(e.to_string()))?
        };

        metadata.modified_at = self.now_iso();
        self.write_metadata(&metadata)?;

        Ok(File {
            id: file_id,
            metadata,
            content,
        })
    }

    fn delete_file(&self, file_id: Uuid) -> Result<(), FileManagerError> {
        let metadata = self.read_metadata(file_id)?;

        // Remove content file
        let file_path = self.file_path(file_id, &metadata.ext);
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| FileManagerError::IoError(e.to_string()))?;
        }

        // Remove metadata file
        let meta_path = self.meta_file_path(file_id);
        if meta_path.exists() {
            fs::remove_file(&meta_path).map_err(|e| FileManagerError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError> {
        let all = self.list_files()?;

        let results: Vec<Metadata> = all
            .into_iter()
            .filter(|m| {
                // Extension filter
                if let Some(ref ext) = query.extension {
                    if !m.ext.eq_ignore_ascii_case(ext) {
                        return false;
                    }
                }

                // Name contains
                if let Some(ref name) = query.name_contains {
                    if !m.name.to_lowercase().contains(&name.to_lowercase()) {
                        return false;
                    }
                }

                // Size filters
                if let Some(min) = query.size_greater_than {
                    if m.size < min {
                        return false;
                    }
                }
                if let Some(max) = query.size_smaller_than {
                    if m.size > max {
                        return false;
                    }
                }

                // Tag filter
                if let Some(ref tags) = query.tags {
                    for tag in tags {
                        if !m.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                            return false;
                        }
                    }
                }

                true
            })
            .collect();

        // Content search (need to read files)
        if query.file_contains.is_some() {
            let search_term = query.file_contains.as_ref().unwrap().to_lowercase();
            let mut content_results = Vec::new();
            for m in &results {
                let file_path = self.file_path(m.id, &m.ext);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if content.to_lowercase().contains(&search_term) {
                        content_results.push(m.clone());
                    }
                }
            }
            return Ok(content_results);
        }

        Ok(results)
    }
}
