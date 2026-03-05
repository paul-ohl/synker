use std::sync::Arc;

use tracing::{debug, instrument, warn};
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
    #[instrument(skip(self), fields(name = %new_file.metadata.name, ext = %new_file.metadata.ext))]
    fn create_file(&self, new_file: NewFile) -> Result<File, FileManagerError> {
        debug!("Validating new file");
        // Business rule: validate the NewFile metadata
        let validated = NewFile::new(new_file.metadata, new_file.content).map_err(|e| {
            warn!(error = %e, "Validation failed for create_file");
            FileManagerError::ValidationError(e.to_string())
        })?;

        let result = self.adapter.create_file(validated);
        if let Ok(ref f) = result {
            debug!(file_id = %f.id, "File created successfully");
        }
        result
    }

    #[instrument(skip(self, data), fields(name, ext))]
    fn create_file_bytes(
        &self,
        name: String,
        ext: String,
        mime: String,
        tags: Vec<String>,
        data: Vec<u8>,
    ) -> Result<Metadata, FileManagerError> {
        debug!(name = %name, ext = %ext, bytes = data.len(), "Validating binary file upload");
        if name.trim().is_empty() {
            warn!("create_file_bytes rejected: empty file name");
            return Err(FileManagerError::ValidationError(
                "File name cannot be empty".to_string(),
            ));
        }
        if ext.trim().is_empty() {
            warn!("create_file_bytes rejected: empty extension");
            return Err(FileManagerError::ValidationError(
                "Extension cannot be empty".to_string(),
            ));
        }
        let result = self.adapter.create_file_bytes(name, ext, mime, tags, data);
        if let Ok(ref m) = result {
            debug!(file_id = %m.id, "Binary file created successfully");
        }
        result
    }

    #[instrument(skip(self))]
    fn read_file(&self, file_id: Uuid) -> Result<File, FileManagerError> {
        debug!("Reading file");
        let result = self.adapter.read_file(file_id);
        if let Err(ref e) = result {
            warn!(file_id = %file_id, error = %e, "read_file failed");
        }
        result
    }

    #[instrument(skip(self))]
    fn read_file_bytes(&self, file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError> {
        debug!("Reading file bytes");
        let result = self.adapter.read_file_bytes(file_id);
        if let Err(ref e) = result {
            warn!(file_id = %file_id, error = %e, "read_file_bytes failed");
        }
        result
    }

    #[instrument(skip(self))]
    fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError> {
        debug!("Listing all files");
        let result = self.adapter.list_files();
        if let Ok(ref files) = result {
            debug!(count = files.len(), "Listed files");
        }
        result
    }

    #[instrument(skip(self))]
    fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError> {
        debug!("Listing all tags");
        let result = self.adapter.list_all_tags();
        if let Ok(ref tags) = result {
            debug!(count = tags.len(), "Listed tags");
        }
        result
    }

    #[instrument(skip(self))]
    fn update_file(&self, file_id: Uuid, update: UpdateFile) -> Result<File, FileManagerError> {
        debug!("Validating file update");
        // Validate update before delegating
        let validated = UpdateFile::new(update.metadata, update.content).map_err(|e| {
            warn!(file_id = %file_id, error = %e, "Validation failed for update_file");
            FileManagerError::ValidationError(e.to_string())
        })?;

        let result = self.adapter.update_file(file_id, validated);
        if let Err(ref e) = result {
            warn!(file_id = %file_id, error = %e, "update_file failed");
        } else {
            debug!(file_id = %file_id, "File updated successfully");
        }
        result
    }

    #[instrument(skip(self))]
    fn delete_file(&self, file_id: Uuid) -> Result<(), FileManagerError> {
        debug!("Deleting file");
        let result = self.adapter.delete_file(file_id);
        if let Err(ref e) = result {
            warn!(file_id = %file_id, error = %e, "delete_file failed");
        } else {
            debug!(file_id = %file_id, "File deleted successfully");
        }
        result
    }

    #[instrument(skip(self))]
    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError> {
        debug!("Executing file query");
        query.verify().map_err(|e| {
            warn!(error = %e, "Invalid file query");
            FileManagerError::ValidationError(e)
        })?;

        let result = self.adapter.find(query);
        if let Ok(ref files) = result {
            debug!(matches = files.len(), "Query returned results");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::file_manager::FileManager;
    use crate::domain::types::file::file_metadata::NewMetadata;
    use std::sync::Mutex;

    // ── Mock adapter ─────────────────────────────────────────────────────────

    struct MockFileManager {
        create_file_result: Mutex<Option<Result<File, FileManagerError>>>,
        create_file_bytes_result: Mutex<Option<Result<Metadata, FileManagerError>>>,
        read_file_result: Mutex<Option<Result<File, FileManagerError>>>,
        read_file_bytes_result: Mutex<Option<Result<(Metadata, Vec<u8>), FileManagerError>>>,
        list_files_result: Mutex<Option<Result<Vec<Metadata>, FileManagerError>>>,
        list_all_tags_result: Mutex<Option<Result<Vec<String>, FileManagerError>>>,
        update_file_result: Mutex<Option<Result<File, FileManagerError>>>,
        delete_file_result: Mutex<Option<Result<(), FileManagerError>>>,
        find_result: Mutex<Option<Result<Vec<Metadata>, FileManagerError>>>,
    }

    impl MockFileManager {
        fn new() -> Self {
            Self {
                create_file_result: Mutex::new(None),
                create_file_bytes_result: Mutex::new(None),
                read_file_result: Mutex::new(None),
                read_file_bytes_result: Mutex::new(None),
                list_files_result: Mutex::new(None),
                list_all_tags_result: Mutex::new(None),
                update_file_result: Mutex::new(None),
                delete_file_result: Mutex::new(None),
                find_result: Mutex::new(None),
            }
        }
    }

    impl ports::FileManager for MockFileManager {
        fn create_file(&self, _new_file: NewFile) -> Result<File, FileManagerError> {
            self.create_file_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn create_file_bytes(
            &self,
            _name: String,
            _ext: String,
            _mime: String,
            _tags: Vec<String>,
            _data: Vec<u8>,
        ) -> Result<Metadata, FileManagerError> {
            self.create_file_bytes_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn read_file(&self, _file_id: Uuid) -> Result<File, FileManagerError> {
            self.read_file_result.lock().unwrap().take().unwrap_or(Err(
                FileManagerError::UnknownError("no mock value set".to_string()),
            ))
        }

        fn read_file_bytes(&self, _file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError> {
            self.read_file_bytes_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError> {
            self.list_files_result.lock().unwrap().take().unwrap_or(Err(
                FileManagerError::UnknownError("no mock value set".to_string()),
            ))
        }

        fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError> {
            self.list_all_tags_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn update_file(
            &self,
            _file_id: Uuid,
            _update: UpdateFile,
        ) -> Result<File, FileManagerError> {
            self.update_file_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn delete_file(&self, _file_id: Uuid) -> Result<(), FileManagerError> {
            self.delete_file_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }

        fn find(&self, _query: FileQuery) -> Result<Vec<Metadata>, FileManagerError> {
            self.find_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Err(FileManagerError::UnknownError(
                    "no mock value set".to_string(),
                )))
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_logic(mock: MockFileManager) -> FileManagerLogic {
        FileManagerLogic::new(Arc::new(mock))
    }

    fn dummy_metadata(id: Uuid) -> Metadata {
        Metadata {
            id,
            name: "test".to_string(),
            ext: "md".to_string(),
            tags: vec![],
            size: 0,
            mime: "text/plain".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            modified_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn dummy_file(id: Uuid) -> File {
        File {
            id,
            metadata: dummy_metadata(id),
            content: "hello".to_string(),
        }
    }

    fn valid_new_file() -> NewFile {
        NewFile {
            metadata: NewMetadata {
                name: "my-file".to_string(),
                ext: "md".to_string(),
                tags: vec![],
                mime: "text/plain".to_string(),
            },
            content: None,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_file_valid() {
        let id = Uuid::new_v4();
        let mock = MockFileManager::new();
        *mock.create_file_result.lock().unwrap() = Some(Ok(dummy_file(id)));
        let logic = make_logic(mock);

        let result = logic.create_file(valid_new_file());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);
    }

    #[test]
    fn test_create_file_invalid_name() {
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        let bad_new_file = NewFile {
            metadata: NewMetadata {
                name: "".to_string(),
                ext: "md".to_string(),
                tags: vec![],
                mime: "text/plain".to_string(),
            },
            content: None,
        };

        let result = logic.create_file(bad_new_file);
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_create_file_invalid_ext() {
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        let bad_new_file = NewFile {
            metadata: NewMetadata {
                name: "readme".to_string(),
                ext: "".to_string(),
                tags: vec![],
                mime: "text/plain".to_string(),
            },
            content: None,
        };

        let result = logic.create_file(bad_new_file);
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_create_file_bytes_empty_name() {
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        let result = logic.create_file_bytes(
            "".to_string(),
            "png".to_string(),
            "image/png".to_string(),
            vec![],
            vec![0u8, 1u8, 2u8],
        );
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_create_file_bytes_empty_ext() {
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        let result = logic.create_file_bytes(
            "photo".to_string(),
            "".to_string(),
            "image/png".to_string(),
            vec![],
            vec![0u8, 1u8, 2u8],
        );
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_update_file_valid() {
        let id = Uuid::new_v4();
        let mock = MockFileManager::new();
        *mock.update_file_result.lock().unwrap() = Some(Ok(dummy_file(id)));
        let logic = make_logic(mock);

        let update = UpdateFile {
            metadata: None,
            content: Some("new content".to_string()),
        };
        let result = logic.update_file(id, update);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_file_invalid_metadata() {
        use crate::domain::types::file::file_metadata::UpdateMetadata;

        let id = Uuid::new_v4();
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        let update = UpdateFile {
            metadata: Some(UpdateMetadata {
                name: Some("".to_string()), // empty name — invalid
                ext: None,
                tags: None,
                mime: None,
            }),
            content: None,
        };
        let result = logic.update_file(id, update);
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_find_valid_query() {
        let mock = MockFileManager::new();
        *mock.find_result.lock().unwrap() = Some(Ok(vec![]));
        let logic = make_logic(mock);

        let query = FileQuery::new().with_extension("rs".to_string());
        let result = logic.find(query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_invalid_query() {
        let mock = MockFileManager::new();
        let logic = make_logic(mock);

        // Conflicting backlinks_to + links_to fails verify()
        let query = FileQuery::new()
            .with_backlinks_to(vec![Uuid::new_v4()])
            .with_links_to(Uuid::new_v4());

        let result = logic.find(query);
        assert!(matches!(result, Err(FileManagerError::ValidationError(_))));
    }

    #[test]
    fn test_read_file_delegates() {
        let id = Uuid::new_v4();
        let mock = MockFileManager::new();
        *mock.read_file_result.lock().unwrap() = Some(Ok(dummy_file(id)));
        let logic = make_logic(mock);

        let result = logic.read_file(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);
    }

    #[test]
    fn test_list_files_delegates() {
        let mock = MockFileManager::new();
        *mock.list_files_result.lock().unwrap() = Some(Ok(vec![]));
        let logic = make_logic(mock);

        let result = logic.list_files();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_delete_file_delegates() {
        let id = Uuid::new_v4();
        let mock = MockFileManager::new();
        *mock.delete_file_result.lock().unwrap() = Some(Ok(()));
        let logic = make_logic(mock);

        let result = logic.delete_file(id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_all_tags_delegates() {
        let mock = MockFileManager::new();
        *mock.list_all_tags_result.lock().unwrap() =
            Some(Ok(vec!["rust".to_string(), "docs".to_string()]));
        let logic = make_logic(mock);

        let result = logic.list_all_tags();
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags, vec!["rust".to_string(), "docs".to_string()]);
    }
}
