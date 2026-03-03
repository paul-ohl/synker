use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "search/search.html")]
pub struct SearchTemplate {
    // Future fields for Askama context:
    // pub available_tags: Vec<String>,
    // pub results: Vec<FileViewModel>,
}

pub async fn search_page() -> Html<String> {
    let tmpl = SearchTemplate {};
    Html(tmpl.render().expect("Failed to render search template"))
}
