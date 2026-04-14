use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use rust_embed::RustEmbed;

/// Static files embedded in the binary.
#[derive(RustEmbed)]
#[folder = "../static/dist/"]
struct StaticFiles;

/// Serves a file from the embedded filesystem.
fn serve_embedded(path: &str) -> Option<Response<Body>> {
    let file = StaticFiles::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(file.data.into_owned()))
        .ok()
}

/// Serves the UI statically.
pub(crate) async fn static_handler(uri: axum::http::Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    serve_embedded(path).unwrap_or_else(|| {
        // SPA fallback — serve index.html for client-side routes
        serve_embedded("index.html").unwrap_or_else(|| {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("UI not built — run `make ui-build`"))
                .unwrap()
        })
    })
}
