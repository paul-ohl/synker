use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use std::env;
use tower_http::services::ServeDir;

use super::handlers::api;
use super::handlers::editor::editor_page;
use super::handlers::search::search_page;
use super::state::AppState;
use crate::domain::logic::file_manager::FileManagerLogic;
use crate::outbound::file_system::FsFileManager;

/// # Panics
///
/// Panics if binding to the address fails or if the server fails to start.
pub async fn server() {
    dotenv().ok();
    let port = env::var("PORT").expect("PORT environment variable not set");
    let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data/files".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Build the dependency graph (hexagonal wiring)
    let fs_adapter =
        FsFileManager::new(&data_dir).expect("Failed to initialise filesystem adapter");
    let file_manager = Arc::new(FileManagerLogic::new(Arc::new(fs_adapter)));

    let state = AppState { file_manager };

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on port {}", port);
    axum::serve(listener, get_routes(state)).await.unwrap();
}

fn get_routes(state: AppState) -> Router<()> {
    Router::new()
        .merge(get_frontend_routes())
        .nest("/api", get_backend_routes(state))
}

fn get_backend_routes(state: AppState) -> Router<()> {
    Router::new()
        .route("/files", post(api::create_file).get(api::list_files))
        .route("/files/upload", post(api::upload_file))
        .route(
            "/files/{id}",
            get(api::get_file)
                .put(api::update_file)
                .delete(api::delete_file),
        )
        .route("/files/{id}/download", get(api::download_file))
        .route("/files/{id}/raw", get(api::serve_file_raw))
        .route("/tags", get(api::list_tags))
        .with_state(state)
}

fn get_frontend_routes() -> Router<()> {
    Router::new()
        .route("/", get(editor_page))
        .route("/files", get(search_page))
        .nest_service("/static", ServeDir::new("static"))
}
