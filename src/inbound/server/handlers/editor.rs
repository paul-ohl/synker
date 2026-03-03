use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "editor/editor.html")]
pub struct EditorTemplate {
    // Future fields for Askama context:
    // pub files: Vec<FileViewModel>,
    // pub active_file_id: Option<String>,
    // pub file_name: String,
}

pub async fn editor_page() -> Html<String> {
    let tmpl = EditorTemplate {};
    Html(tmpl.render().expect("Failed to render editor template"))
}
