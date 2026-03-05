use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
    routing::{get, post},
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use synker::{
    domain::logic::file_manager::FileManagerLogic,
    inbound::server::{
        handlers::backend::{
            create_file::create_file,
            delete_file::delete_file,
            download_file::download_file,
            get_file::get_file,
            list_files::list_files,
            list_tags::list_tags,
            serve_file_raw::serve_file_raw,
            update_file::update_file,
            upload_file::upload_file,
        },
        state::AppState,
    },
    outbound::file_system::FsFileManager,
};
use tempfile::tempdir;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_test_router() -> (Router, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let fs_adapter = FsFileManager::new(dir.path()).unwrap();
    let file_manager = Arc::new(FileManagerLogic::new(Arc::new(fs_adapter)));
    let state = AppState { file_manager };

    let router = Router::new()
        .route("/api/files", post(create_file).get(list_files))
        .route(
            "/api/files/{id}",
            get(get_file).put(update_file).delete(delete_file),
        )
        .route("/api/files/{id}/download", get(download_file))
        .route("/api/files/{id}/raw", get(serve_file_raw))
        .route("/api/files/upload", post(upload_file))
        .route("/api/tags", get(list_tags))
        .with_state(state);

    (router, dir)
}

async fn body_to_json(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    let body_str = body.to_string();
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body_str))
        .unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_file_returns_201() {
    let (router, _dir) = build_test_router();

    let req = json_request(
        "POST",
        "/api/files",
        json!({
            "name": "hello",
            "ext": "md",
            "mime": "text/markdown",
            "content": "# Hello World"
        }),
    );

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = body_to_json(resp.into_body()).await;
    assert_eq!(body["name"], "hello");
    assert_eq!(body["ext"], "md");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_file_invalid_returns_400() {
    let (router, _dir) = build_test_router();

    // Empty name should fail validation
    let req = json_request(
        "POST",
        "/api/files",
        json!({
            "name": "",
            "ext": "md",
            "mime": "text/markdown"
        }),
    );

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_files_returns_200() {
    let (router, _dir) = build_test_router();

    let req = Request::builder()
        .method("GET")
        .uri("/api/files")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_to_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[tokio::test]
async fn test_list_files_returns_all_created() {
    let (router, _dir) = build_test_router();

    // Create two files then list
    for name in &["alpha", "beta"] {
        let req = json_request(
            "POST",
            "/api/files",
            json!({ "name": name, "ext": "txt", "mime": "text/plain" }),
        );
        let resp = router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let list_req = Request::builder()
        .method("GET")
        .uri("/api/files")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_to_json(resp.into_body()).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[tokio::test]
async fn test_get_file_returns_200() {
    let (router, _dir) = build_test_router();

    // Create a file first
    let create_req = json_request(
        "POST",
        "/api/files",
        json!({ "name": "greet", "ext": "md", "mime": "text/markdown", "content": "hello" }),
    );
    let create_resp = router.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let created = body_to_json(create_resp.into_body()).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Now GET it
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{id}"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_to_json(resp.into_body()).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "greet");
}

#[tokio::test]
async fn test_get_file_not_found_returns_404() {
    let (router, _dir) = build_test_router();

    let fake_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{fake_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_file_returns_200() {
    let (router, _dir) = build_test_router();

    // Create
    let create_req = json_request(
        "POST",
        "/api/files",
        json!({ "name": "doc", "ext": "md", "mime": "text/markdown", "content": "old" }),
    );
    let create_resp = router.clone().oneshot(create_req).await.unwrap();
    let created = body_to_json(create_resp.into_body()).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Update content
    let update_req = json_request(
        "PUT",
        &format!("/api/files/{id}"),
        json!({ "content": "new content" }),
    );
    let resp = router.oneshot(update_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_to_json(resp.into_body()).await;
    assert_eq!(body["content"], "new content");
}

#[tokio::test]
async fn test_delete_file_returns_204() {
    let (router, _dir) = build_test_router();

    // Create
    let create_req = json_request(
        "POST",
        "/api/files",
        json!({ "name": "to-delete", "ext": "md", "mime": "text/plain" }),
    );
    let create_resp = router.clone().oneshot(create_req).await.unwrap();
    let created = body_to_json(create_resp.into_body()).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Delete
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/files/{id}"))
        .body(Body::empty())
        .unwrap();

    let resp = router.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Confirm it's gone
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{id}"))
        .body(Body::empty())
        .unwrap();
    let get_resp = router.oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_tags_returns_200() {
    let (router, _dir) = build_test_router();

    // Create some tagged files
    let req1 = json_request(
        "POST",
        "/api/files",
        json!({ "name": "f1", "ext": "md", "mime": "text/plain", "tags": ["rust", "docs"] }),
    );
    let req2 = json_request(
        "POST",
        "/api/files",
        json!({ "name": "f2", "ext": "md", "mime": "text/plain", "tags": ["docs", "code"] }),
    );
    router.clone().oneshot(req1).await.unwrap();
    router.clone().oneshot(req2).await.unwrap();

    let tags_req = Request::builder()
        .method("GET")
        .uri("/api/tags")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(tags_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_to_json(resp.into_body()).await;
    let tags = body.as_array().unwrap();
    // Should have code, docs, rust (deduplicated, sorted)
    assert_eq!(tags.len(), 3);
    let tag_strs: Vec<&str> = tags.iter().map(|t| t.as_str().unwrap()).collect();
    assert!(tag_strs.contains(&"rust"));
    assert!(tag_strs.contains(&"docs"));
    assert!(tag_strs.contains(&"code"));
}

#[tokio::test]
async fn test_download_file_returns_200_with_attachment_header() {
    let (router, _dir) = build_test_router();

    // Create a file first
    let create_req = json_request(
        "POST",
        "/api/files",
        json!({ "name": "report", "ext": "txt", "mime": "text/plain", "content": "download me" }),
    );
    let create_resp = router.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let created = body_to_json(create_resp.into_body()).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Download it
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{id}/download"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_disp.contains("attachment"),
        "Expected content-disposition to contain 'attachment', got: {content_disp}"
    );
    assert!(content_disp.contains("report.txt"));
}

#[tokio::test]
async fn test_download_file_not_found_returns_404() {
    let (router, _dir) = build_test_router();

    let fake_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{fake_id}/download"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_serve_file_raw_returns_200_inline() {
    let (router, _dir) = build_test_router();

    // Create a file
    let create_req = json_request(
        "POST",
        "/api/files",
        json!({ "name": "image", "ext": "png", "mime": "image/png", "content": "fake-png-data" }),
    );
    let create_resp = router.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let created = body_to_json(create_resp.into_body()).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Serve raw
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{id}/raw"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("image/png"),
        "Expected content-type image/png, got: {content_type}"
    );
}

#[tokio::test]
async fn test_serve_file_raw_not_found_returns_404() {
    let (router, _dir) = build_test_router();

    let fake_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/files/{fake_id}/raw"))
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_upload_file_returns_201() {
    let (router, _dir) = build_test_router();

    // Build a minimal multipart body: boundary + file field
    let boundary = "testboundary1234";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\
         \r\n\
         hello upload\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .method("POST")
        .uri("/api/files/upload")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["ext"], "txt");
}

#[tokio::test]
async fn test_upload_file_with_tags_returns_201() {
    let (router, _dir) = build_test_router();

    let boundary = "testboundary5678";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"notes.md\"\r\n\
         Content-Type: text/markdown\r\n\
         \r\n\
         # Notes content\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"tags\"\r\n\
         \r\n\
         rust,docs\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .method("POST")
        .uri("/api/files/upload")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["name"], "notes");
    assert_eq!(json["ext"], "md");
    let tags = json["tags"].as_array().unwrap();
    let tag_strs: Vec<&str> = tags.iter().map(|t| t.as_str().unwrap()).collect();
    assert!(tag_strs.contains(&"rust"));
    assert!(tag_strs.contains(&"docs"));
}

#[tokio::test]
async fn test_upload_file_no_file_field_returns_400() {
    let (router, _dir) = build_test_router();

    // Multipart with only a tags field, no "file" field
    let boundary = "testboundary9999";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"tags\"\r\n\
         \r\n\
         rust\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .method("POST")
        .uri("/api/files/upload")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_file_no_extension_defaults_to_bin() {
    let (router, _dir) = build_test_router();

    let boundary = "testboundarynoext";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"noextension\"\r\n\
         Content-Type: application/octet-stream\r\n\
         \r\n\
         raw bytes\r\n\
         --{boundary}--\r\n"
    );

    let req = Request::builder()
        .method("POST")
        .uri("/api/files/upload")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&resp_body).unwrap();
    // filename has no dot → name="noextension", ext defaults to "bin"
    assert_eq!(json["ext"], "bin");
}
