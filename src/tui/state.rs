use std::sync::Arc;

use crate::{auth::LoginFlow, http::HttpClient};

#[derive(Debug)]
pub struct TuiState {
    pub login: LoginFlow,
    pub tick: usize,
}

impl TuiState {
    pub fn create(client: Arc<HttpClient>) -> Self {
        Self {
            login: LoginFlow::create(client),
            tick: usize::default(),
        }
    }

    pub async fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.login.tick().await;
    }
}
