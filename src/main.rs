mod app;
mod auth;
mod error;
mod result;
mod tui;
mod mods;

#[tokio::main]
async fn main() -> result::Result<()> {
    app::run().await
}
