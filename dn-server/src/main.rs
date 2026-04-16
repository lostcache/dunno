mod args;
mod route;
mod router;
mod schema;
mod server;
mod service;
mod surreal;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::Server::start().await?;
    Ok(())
}
