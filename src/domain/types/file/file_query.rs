use std::time::SystemTime;

use uuid::Uuid;

/// A flexible query builder for filtering and searching files.
///
/// `FileQuery` allows you to construct complex file search queries with support for multiple
/// filtering criteria including file properties (extension, name, size), content, modification
/// dates, tags, and link relationships.
///
/// # Fields
///
/// * `extension` - Filter by file extension (e.g., "rs", "md")
/// * `name_contains` - Filter by partial name match
/// * `file_contains` - Filter by file content substring
/// * `size_greater_than` - Filter files larger than this size in bytes
/// * `size_smaller_than` - Filter files smaller than this size in bytes
/// * `tags` - Filter by associated tags
/// * `modified_after` - Filter files modified after this time
/// * `modified_before` - Filter files modified before this time
/// * `orphans` - Filter for orphaned files (files with no backlinks)
/// * `backlinks_to` - Filter for files that have backlinks to specific file IDs
/// * `links_to` - Filter for files that link to a specific file ID
#[derive(Debug, Clone)]
pub struct FileQuery {
    pub extension: Option<String>,
    pub name_contains: Option<String>,
    pub file_contains: Option<String>,
    pub size_greater_than: Option<u64>,
    pub size_smaller_than: Option<u64>,
    pub tags: Option<Vec<String>>,
    pub modified_after: Option<SystemTime>,
    pub modified_before: Option<SystemTime>,
    pub orphans: bool,
    pub backlinks_to: Option<Vec<Uuid>>,
    pub links_to: Option<Uuid>,
}

impl FileQuery {
    /// Creates a new empty `FileQuery` with all filters unset.
    ///
    /// # Examples
    ///
    /// ```
    /// let query = FileQuery::new();
    /// ```
    pub fn new() -> Self {
        Self {
            extension: None,
            name_contains: None,
            file_contains: None,
            size_greater_than: None,
            size_smaller_than: None,
            tags: None,
            modified_after: None,
            modified_before: None,
            orphans: false,
            backlinks_to: None,
            links_to: None,
        }
    }

    /// Sets the file extension filter.
    ///
    /// # Arguments
    ///
    /// * `extension` - The file extension to filter by (e.g., "rs", "md")
    pub fn with_extension(mut self, extension: String) -> Self {
        self.extension = Some(extension);
        self
    }

    /// Sets the name substring filter.
    ///
    /// # Arguments
    ///
    /// * `name` - The substring that filenames must contain
    pub fn with_name_contains(mut self, name: String) -> Self {
        self.name_contains = Some(name);
        self
    }

    /// Sets the file content filter.
    ///
    /// # Arguments
    ///
    /// * `content` - The substring that file contents must contain
    pub fn with_file_contains(mut self, content: String) -> Self {
        self.file_contains = Some(content);
        self
    }

    /// Sets the minimum file size filter (inclusive).
    ///
    /// # Arguments
    ///
    /// * `size` - Minimum file size in bytes
    pub fn with_size_greater_than(mut self, size: u64) -> Self {
        self.size_greater_than = Some(size);
        self
    }

    /// Sets the maximum file size filter (inclusive).
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum file size in bytes
    pub fn with_size_smaller_than(mut self, size: u64) -> Self {
        self.size_smaller_than = Some(size);
        self
    }

    /// Sets the tags filter.
    ///
    /// # Arguments
    ///
    /// * `tags` - Vector of tags the file must have
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the modification time start filter (inclusive).
    ///
    /// # Arguments
    ///
    /// * `time` - Files must be modified after this time
    pub fn with_modified_after(mut self, time: SystemTime) -> Self {
        self.modified_after = Some(time);
        self
    }

    /// Sets the modification time end filter (inclusive).
    ///
    /// # Arguments
    ///
    /// * `time` - Files must be modified before this time
    pub fn with_modified_before(mut self, time: SystemTime) -> Self {
        self.modified_before = Some(time);
        self
    }

    /// Enables or disables the orphaned files filter.
    ///
    /// When set to `true`, only files with no incoming backlinks are returned.
    ///
    /// # Arguments
    ///
    /// * `orphans` - Whether to filter for orphaned files
    pub fn with_orphans(mut self, orphans: bool) -> Self {
        self.orphans = orphans;
        self
    }

    /// Sets the backlinks filter to specific file IDs.
    ///
    /// Returns files that have backlinks to any of the provided file IDs.
    ///
    /// # Arguments
    ///
    /// * `ids` - Vector of file IDs to filter backlinks by
    ///
    /// # Panics
    ///
    /// Will fail validation if used together with [`with_links_to`](Self::with_links_to).
    pub fn with_backlinks_to(mut self, ids: Vec<Uuid>) -> Self {
        self.backlinks_to = Some(ids);
        self
    }

    /// Sets the outgoing links filter to a specific file ID.
    ///
    /// Returns files that link to the provided file ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The file ID to filter links by
    ///
    /// # Panics
    ///
    /// Will fail validation if used together with [`with_backlinks_to`](Self::with_backlinks_to).
    pub fn with_links_to(mut self, id: Uuid) -> Self {
        self.links_to = Some(id);
        self
    }

    /// Validates the query for logical consistency.
    ///
    /// This method ensures all filter combinations are valid:
    /// - `size_greater_than` must not exceed `size_smaller_than`
    /// - `modified_after` must not be later than `modified_before`
    /// - `backlinks_to` and `links_to` cannot both be set
    ///
    /// # Errors
    ///
    /// Returns a descriptive error message if any validation rule is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// let query = FileQuery::new()
    ///     .with_extension("rs".to_string())
    ///     .verify()?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn verify(&self) -> Result<(), String> {
        if let (Some(greater), Some(smaller)) = (self.size_greater_than, self.size_smaller_than) {
            if greater > smaller {
                return Err("size_greater_than cannot be larger than size_smaller_than".to_string());
            }
        }

        if let (Some(after), Some(before)) = (self.modified_after, self.modified_before) {
            if after > before {
                return Err("modified_after cannot be later than modified_before".to_string());
            }
        }

        if self.backlinks_to.is_some() && self.links_to.is_some() {
            return Err("backlinks_to and links_to cannot be used at the same time".to_string());
        }

        Ok(())
    }
}

impl Default for FileQuery {
    /// Creates a default `FileQuery` (equivalent to [`new()`](Self::new)).
    fn default() -> Self {
        Self::new()
    }
}
