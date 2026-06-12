use crate::{auth::LoginFlow, http::HttpClient};

#[derive(Debug)]
pub struct TuiState<'a> {
    pub login: LoginFlow<'a>,
    pub tick: usize,
}

impl<'a> TuiState<'a> {
    pub fn create(client: &'a HttpClient) -> Self {
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
