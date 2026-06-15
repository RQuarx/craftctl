mod app;
mod auth;
mod config;
mod download;
mod error;
mod http;
mod instances;
mod mods;
mod provider;
mod result;
mod storage;
mod tui;

#[tokio::main]
async fn main() -> result::Result<()> {
    app::run().await
}
