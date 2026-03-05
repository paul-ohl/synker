use std::fs;
use std::io;
use std::path::PathBuf;

use uuid::Uuid;

use crate::domain::{ports, services::file_manager::FileManagerError};
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

impl ports::FileManager for FsFileManager {
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
                if let Some(ref ext) = query.extension
                    && !m.ext.eq_ignore_ascii_case(ext)
                {
                    return false;
                }

                // Name contains
                if let Some(ref name) = query.name_contains
                    && !m.name.to_lowercase().contains(&name.to_lowercase())
                {
                    return false;
                }

                // Size filters
                if let Some(min) = query.size_greater_than
                    && m.size < min
                {
                    return false;
                }
                if let Some(max) = query.size_smaller_than
                    && m.size > max
                {
                    return false;
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
        if let Some(file_contains) = &query.file_contains {
            let search_term = file_contains.to_lowercase();
            let mut content_results = Vec::new();
            for m in &results {
                let file_path = self.file_path(m.id, &m.ext);
                if let Ok(content) = fs::read_to_string(&file_path)
                    && content.to_lowercase().contains(&search_term)
                {
                    content_results.push(m.clone());
                }
            }
            return Ok(content_results);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Bring the FileManager port trait into scope so we can call its methods on FsFileManager
    use crate::domain::ports::FileManager as _;
    use crate::domain::types::file::file_metadata::{NewMetadata, UpdateMetadata};
    use tempfile::tempdir;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_fs() -> (FsFileManager, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let fs = FsFileManager::new(dir.path()).unwrap();
        (fs, dir)
    }

    fn create_test_file(
        fs: &FsFileManager,
        name: &str,
        ext: &str,
        content: &str,
        tags: Vec<String>,
    ) -> crate::domain::types::file::file::File {
        let new_file = NewFile {
            metadata: NewMetadata {
                name: name.to_string(),
                ext: ext.to_string(),
                tags,
                mime: "text/plain".to_string(),
            },
            content: Some(content.to_string()),
        };
        fs.create_file(new_file).unwrap()
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_and_read_file() {
        let (fs, _dir) = make_fs();
        let file = create_test_file(&fs, "readme", "md", "# Hello", vec![]);

        let read_back = fs.read_file(file.id).unwrap();
        assert_eq!(read_back.id, file.id);
        assert_eq!(read_back.metadata.name, "readme");
        assert_eq!(read_back.metadata.ext, "md");
        assert_eq!(read_back.content, "# Hello");
        assert_eq!(read_back.metadata.size, 7); // "# Hello" is 7 bytes
    }

    #[test]
    fn test_create_file_bytes_and_read() {
        let (fs, _dir) = make_fs();
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let metadata = fs
            .create_file_bytes(
                "image".to_string(),
                "bin".to_string(),
                "application/octet-stream".to_string(),
                vec![],
                data.clone(),
            )
            .unwrap();

        let (read_meta, read_data) = fs.read_file_bytes(metadata.id).unwrap();
        assert_eq!(read_meta.name, "image");
        assert_eq!(read_meta.ext, "bin");
        assert_eq!(read_meta.size, 4);
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let (fs, _dir) = make_fs();
        let random_id = Uuid::new_v4();
        let result = fs.read_file(random_id);
        assert!(matches!(result, Err(FileManagerError::FileNotFound(_))));
    }

    #[test]
    fn test_list_files_empty() {
        let (fs, _dir) = make_fs();
        let files = fs.list_files().unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_list_files_multiple() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "alpha", "md", "a", vec![]);
        // Sleep >1 second to ensure distinct RFC3339 second-level timestamps
        std::thread::sleep(std::time::Duration::from_millis(1100));
        create_test_file(&fs, "beta", "rs", "b", vec![]);
        std::thread::sleep(std::time::Duration::from_millis(1100));
        create_test_file(&fs, "gamma", "txt", "c", vec![]);

        let files = fs.list_files().unwrap();
        assert_eq!(files.len(), 3);
        // sorted newest first
        assert_eq!(files[0].name, "gamma");
        assert_eq!(files[2].name, "alpha");
    }

    #[test]
    fn test_list_all_tags_deduplication() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "f1", "md", "", vec!["rust".to_string(), "docs".to_string()]);
        create_test_file(&fs, "f2", "md", "", vec!["docs".to_string(), "code".to_string()]);
        create_test_file(&fs, "f3", "md", "", vec!["rust".to_string()]);

        let tags = fs.list_all_tags().unwrap();
        // BTreeSet ensures sorted, deduplicated
        assert_eq!(tags, vec!["code", "docs", "rust"]);
    }

    #[test]
    fn test_update_file_content_only() {
        let (fs, _dir) = make_fs();
        let file = create_test_file(&fs, "note", "txt", "old content", vec![]);
        let original_name = file.metadata.name.clone();

        let update = UpdateFile {
            metadata: None,
            content: Some("new content".to_string()),
        };
        let updated = fs.update_file(file.id, update).unwrap();

        assert_eq!(updated.content, "new content");
        assert_eq!(updated.metadata.name, original_name); // name unchanged
        assert_eq!(updated.metadata.size, 11); // "new content" length
    }

    #[test]
    fn test_update_file_metadata_only() {
        let (fs, _dir) = make_fs();
        let file = create_test_file(&fs, "old-name", "md", "content here", vec![]);

        let update = UpdateFile {
            metadata: Some(UpdateMetadata {
                name: Some("new-name".to_string()),
                ext: None,
                tags: None,
                mime: None,
            }),
            content: None,
        };
        let updated = fs.update_file(file.id, update).unwrap();

        assert_eq!(updated.metadata.name, "new-name");
        assert_eq!(updated.content, "content here"); // content unchanged
    }

    #[test]
    fn test_update_file_extension_rename() {
        let (fs, _dir) = make_fs();
        let file = create_test_file(&fs, "script", "txt", "print('hi')", vec![]);

        // Check old file exists
        let old_path = fs.base_path.join(format!("{}.txt", file.id));
        assert!(old_path.exists());

        let update = UpdateFile {
            metadata: Some(UpdateMetadata {
                name: None,
                ext: Some("py".to_string()),
                tags: None,
                mime: None,
            }),
            content: None,
        };
        let updated = fs.update_file(file.id, update).unwrap();

        assert_eq!(updated.metadata.ext, "py");
        // Old file should be gone, new file should exist
        assert!(!old_path.exists());
        let new_path = fs.base_path.join(format!("{}.py", file.id));
        assert!(new_path.exists());
    }

    #[test]
    fn test_delete_file() {
        let (fs, _dir) = make_fs();
        let file = create_test_file(&fs, "to-delete", "md", "bye", vec![]);
        let file_id = file.id;

        fs.delete_file(file_id).unwrap();

        // Both content and metadata files should be gone
        let result = fs.read_file(file_id);
        assert!(matches!(result, Err(FileManagerError::FileNotFound(_))));
    }

    #[test]
    fn test_delete_nonexistent() {
        let (fs, _dir) = make_fs();
        let result = fs.delete_file(Uuid::new_v4());
        assert!(matches!(result, Err(FileManagerError::FileNotFound(_))));
    }

    #[test]
    fn test_find_by_extension() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "a", "md", "", vec![]);
        create_test_file(&fs, "b", "rs", "", vec![]);
        create_test_file(&fs, "c", "md", "", vec![]);

        let query = FileQuery::new().with_extension("md".to_string());
        let results = fs.find(query).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|m| m.ext == "md"));
    }

    #[test]
    fn test_find_by_name_contains() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "my-readme", "md", "", vec![]);
        create_test_file(&fs, "changelog", "md", "", vec![]);
        create_test_file(&fs, "MY-NOTES", "md", "", vec![]);

        // case-insensitive match: "my" should match "my-readme" and "MY-NOTES"
        let query = FileQuery::new().with_name_contains("my".to_string());
        let results = fs.find(query).unwrap();

        assert_eq!(results.len(), 2);
        let names: Vec<&str> = results.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"my-readme"));
        assert!(names.contains(&"MY-NOTES"));
    }

    #[test]
    fn test_find_by_tags() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "f1", "md", "", vec!["rust".to_string()]);
        create_test_file(&fs, "f2", "md", "", vec!["rust".to_string(), "docs".to_string()]);
        create_test_file(&fs, "f3", "md", "", vec!["docs".to_string()]);

        // Files with the "rust" tag
        let query = FileQuery::new().with_tags(vec!["rust".to_string()]);
        let results = fs.find(query).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|m| m.tags.contains(&"rust".to_string())));
    }

    #[test]
    fn test_find_by_size() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "tiny", "md", "hi", vec![]); // 2 bytes
        create_test_file(&fs, "medium", "md", "hello world", vec![]); // 11 bytes
        create_test_file(&fs, "large", "md", "a".repeat(100).as_str(), vec![]); // 100 bytes

        let query = FileQuery::new()
            .with_size_greater_than(5)
            .with_size_smaller_than(50);
        let results = fs.find(query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "medium");
    }

    #[test]
    fn test_find_by_content() {
        let (fs, _dir) = make_fs();

        create_test_file(&fs, "has-keyword", "md", "This contains the magic word", vec![]);
        create_test_file(&fs, "no-keyword", "md", "Nothing special here", vec![]);

        let query = FileQuery::new().with_file_contains("magic".to_string());
        let results = fs.find(query).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "has-keyword");
    }
}
