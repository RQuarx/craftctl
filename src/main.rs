mod app;
mod auth;
mod error;
mod mods;
mod result;
mod tui;

#[tokio::main]
async fn main() -> result::Result<()> {
    app::run().await
}
