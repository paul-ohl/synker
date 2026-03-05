use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use super::handlers::backend;
use super::handlers::editor::editor_page;
use super::handlers::search::search_page;
use super::state::AppState;

/// # Panics
///
/// Panics if binding to the address fails or if the server fails to start.
pub async fn server(state: AppState, addr: &str) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, get_routes(state)).await.unwrap();
}

fn get_routes(state: AppState) -> Router<()> {
    Router::new()
        .merge(get_frontend_routes())
        .nest("/api", get_backend_routes(state))
}

fn get_backend_routes(state: AppState) -> Router<()> {
    Router::new()
        .route(
            "/files",
            post(backend::create_file::create_file).get(backend::list_files::list_files),
        )
        .route("/files/upload", post(backend::upload_file::upload_file))
        .route(
            "/files/{id}",
            get(backend::get_file::get_file)
                .put(backend::update_file::update_file)
                .delete(backend::delete_file::delete_file),
        )
        .route(
            "/files/{id}/download",
            get(backend::download_file::download_file),
        )
        .route(
            "/files/{id}/raw",
            get(backend::serve_file_raw::serve_file_raw),
        )
        .route("/tags", get(backend::list_tags::list_tags))
        .with_state(state)
}

fn get_frontend_routes() -> Router<()> {
    Router::new()
        .route("/", get(editor_page))
        .route("/files", get(search_page))
        .nest_service("/static", ServeDir::new("static"))
}
