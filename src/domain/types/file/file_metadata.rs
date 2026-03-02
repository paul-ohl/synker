use uuid::Uuid;

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

pub struct NewMetadata {
    pub name: String,
    pub ext: String,
    pub tags: Vec<String>,
    pub mime: String,
}

pub struct UpdateMetadata {
    pub name: Option<String>,
    pub ext: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime: Option<String>,
}
