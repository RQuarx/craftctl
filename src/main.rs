mod app;
mod auth;
mod download;
mod error;
mod http;
mod mods;
mod provider;
mod result;
mod tui;

#[tokio::main]
async fn main() -> result::Result<()> {
    app::run().await
}
