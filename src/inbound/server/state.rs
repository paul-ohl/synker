use std::sync::Arc;

use crate::domain::logic::file_manager::FileManagerLogic;

/// Shared application state, injected into Axum handlers.
/// Holds domain-layer orchestrators behind Arc for thread-safe sharing.
#[derive(Clone)]
pub struct AppState {
    pub file_manager: Arc<FileManagerLogic>,
}
