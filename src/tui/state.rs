use crate::auth::LoginFlow;

#[derive(Debug, Default)]
pub struct TuiState {
    pub login: LoginFlow,
    pub tick: usize,
}

impl TuiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.login.tick().await;
    }
}
