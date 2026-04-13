mod router;
mod route;
mod schema;
mod service;

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use clap::Parser;
use dn_core::{config::Config, db::surreal::DB};
use rust_embed::RustEmbed;
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "../static/dist/"]
struct StaticFiles;

fn serve_embedded(path: &str) -> Option<Response<Body>> {
    let file = StaticFiles::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(file.data.into_owned()))
        .ok()
}

async fn static_handler(uri: axum::http::Uri) -> Response<Body> {
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

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 7700)]
    port: u16,
    #[arg(long)]
    no_open: bool,
    #[arg(long)]
    backend: Option<String>,
}

pub(crate) struct AppState {
    db: DB,
}

/// Resolves to () when Ctrl+C is received.
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}

fn find_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind :0");
    listener.local_addr().unwrap().port()
}

async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    false
}

fn spawn_surreal_server(
    surreal_port: u16,
    db_path: &std::path::Path,
) -> anyhow::Result<std::process::Child> {
    std::process::Command::new("surreal")
        .args([
            "start",
            "--bind",
            &format!("127.0.0.1:{}", surreal_port),
            "--username",
            "root",
            "--password",
            "root",
            &format!("surrealkv://{}", db_path.to_string_lossy()),
        ])
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "surreal binary not found — install it with:\n  curl -sSf https://install.surrealdb.com | sh"
                )
            } else {
                e.into()
            }
        })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    let config = Config::load(args.backend.as_deref())?;

    let (db, surreal_child): (DB, Option<std::process::Child>) = if matches!(
        config.backend,
        dn_core::config::StorageBackend::Local
    ) {
        let db_path = config.local_data_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let surreal_port = find_free_port();
        match spawn_surreal_server(surreal_port, &db_path) {
            Ok(child) => {
                if !wait_for_port(surreal_port, 10).await {
                    return Err(anyhow::anyhow!(
                        "surreal server did not start within 10 seconds"
                    ));
                }
                let ws_url = format!("ws://127.0.0.1:{}/rpc", surreal_port);
                let db = DB::new(&ws_url).await?;
                (db, Some(child))
            }
            Err(e) => {
                eprintln!("Warning: {e}");
                eprintln!("dn-server started, but the dn CLI will not be available concurrently.");
                eprintln!("To use both at the same time, either:");
                eprintln!(
                    "  - Install surreal and restart dn-server: curl -sSf https://install.surrealdb.com | sh"
                );
                eprintln!(
                    "  - Or set backend = \"local-server\" in your config and run SurrealDB separately"
                );
                let db = DB::from_config(&config).await?;
                (db, None)
            }
        }
    } else {
        let db = DB::from_config(&config).await?;
        (db, None)
    };

    let state = Arc::new(AppState { db });

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let url = format!("http://{}", addr);
    println!("dn-server running at {}", url);

    if !args.no_open {
        open::that(&url).ok();
    }

    axum::serve(listener, router::build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if let Some(mut child) = surreal_child {
        let _ = child.kill();
    }

    Ok(())
}

#[cfg(test)]
mod handler_tests {
    use super::*;

    #[test]
    fn create_todo_body_requires_project_id() {
        let json = r#"{"content":"test"}"#;
        let result: Result<schema::CreateTodoBody, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "project_id must be required — missing field should fail deserialization"
        );
    }

    #[test]
    fn create_todo_body_accepts_valid_body() {
        let json = r#"{"content":"test","project_id":"project:abc"}"#;
        let body: schema::CreateTodoBody =
            serde_json::from_str(json).expect("valid body must deserialize");
        assert_eq!(body.project_id, "project:abc");
        assert_eq!(body.content, "test");
    }
}
