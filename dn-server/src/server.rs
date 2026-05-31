use crate::router;
use clap::Parser;
use dn_core::{config::Config, db::surreal::DB};
use std::sync::Arc;

pub(crate) struct AppState {
    pub(crate) db: DB,
}

pub(crate) struct Server {}

impl Server {
    async fn shutdown_signal() {
        tokio::signal::ctrl_c().await.ok();
    }

    pub(crate) async fn start() -> anyhow::Result<()> {
        let args = crate::args::Args::parse();

        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .ok();

        let config = Config::load()?;

        if matches!(config.backend, dn_core::config::StorageBackend::Embedded) {
            panic!("Cannot use embedded backend with dn-server, use local or cloud instead");
        }

        let db = DB::from_config(&config).await?;

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

        Ok(())
    }
}
