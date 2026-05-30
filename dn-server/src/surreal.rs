use crate::server::Server;
use dn_core::{config::Config, db::surreal::DB};

pub(crate) struct Surreal {}

impl Surreal {
    fn spawn_surreal_process(
        db_path: &std::path::Path,
    ) -> anyhow::Result<(u16, std::process::Child)> {
        let surreal_port = Server::find_free_port();
        let child = std::process::Command::new("surreal")
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
        })?;
        Ok((surreal_port, child))
    }

    pub(crate) async fn spawn_surreal_server(
        db_path: &std::path::Path,
        config: &Config,
    ) -> anyhow::Result<(DB, Option<std::process::Child>)> {
        match Self::spawn_surreal_process(db_path) {
            Ok((surreal_port, child)) => {
                if !Server::wait_for_port(surreal_port, 10).await {
                    return Err(anyhow::anyhow!(
                        "surreal server did not start within 10 seconds"
                    ));
                }
                let ws_url = format!("ws://127.0.0.1:{}/rpc", surreal_port);
                let db = DB::new(&ws_url).await?;
                Ok((db, Some(child)))
            }
            Err(e) => {
                eprintln!("Warning: {e}");
                eprintln!("dn-server started, but the dn CLI will not be available concurrently.");
                eprintln!("To use both at the same time, either:");
                eprintln!(
                    "  - Install surreal and restart dn-server: curl -sSf https://install.surrealdb.com | sh"
                );
                eprintln!(
                    "  - Or set backend = \"local\" in your config and run SurrealDB separately"
                );
                let db = DB::from_config(config).await?;
                Ok((db, None))
            }
        }
    }
}
