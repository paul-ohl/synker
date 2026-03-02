use std::time::SystemTime;

use uuid::Uuid;

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
