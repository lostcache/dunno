use crate::router;
use crate::surreal::Surreal;
use clap::Parser;
use dn_core::{config::Config, db::surreal::DB};
use std::sync::Arc;

pub(crate) struct AppState {
    pub(crate) db: DB,
}

pub(crate) struct Server {}

impl Server {
    /// Finds a free port to run the surreal server on.
    pub(crate) fn find_free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind :0");
        listener.local_addr().unwrap().port()
    }

    /// Resolves to () when Ctrl+C is received.
    async fn shutdown_signal() {
        tokio::signal::ctrl_c().await.ok();
    }

    /// Waits for the surreal server to start listening on the given port.
    pub(crate) async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
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

    pub(crate) async fn start() -> anyhow::Result<()> {
        let args = crate::args::Args::parse();

        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .ok();

        let config = Config::load()?;

        let (db, db_process): (DB, Option<std::process::Child>) =
            if matches!(config.backend, dn_core::config::StorageBackend::Local) {
                let db_path = config.local_data_path();
                if let Some(parent) = db_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                Surreal::spawn_surreal_server(&db_path, &config).await?
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
            .with_graceful_shutdown(Self::shutdown_signal())
            .await?;

        if let Some(mut child) = db_process {
            let _ = child.kill();
        }

        Ok(())
    }
}
