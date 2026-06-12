use std::{
    env,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    auth::{
        Account,
        browser::open,
        create_offline_account,
        microsoft::{MicrosoftAuth, MicrosoftLogin},
    },
    http::HttpClient,
};

const DEFAULT_MICROSOFT_CLIENT_ID: &str = "00000000402b5328";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginPhase {
    Choosing,
    WaitingForMicrosoft,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginClickTarget {
    Microsoft,
    Offline,
}

#[derive(Debug)]
pub struct LoginFlow<'a> {
    client: &'a HttpClient,

    mode: LoginMode,
    phase: LoginPhase,
    username: String,
    account: Option<Account>,
    error: Option<String>,
    login: Option<MicrosoftLogin<'a>>,
    last_poll: Option<Instant>,
}

impl<'a> LoginFlow<'a> {
    pub fn create(client: &'a HttpClient) -> Self {
        Self {
            client,
            mode: LoginMode::Microsoft,
            phase: LoginPhase::Choosing,
            username: String::new(),
            account: None,
            error: None,
            login: None,
            last_poll: None,
        }
    }

    pub fn mode(&self) -> LoginMode {
        self.mode
    }

    pub fn phase(&self) -> LoginPhase {
        self.phase
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn account(&self) -> Option<&Account> {
        self.account.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn microsoft_login(&self) -> Option<&MicrosoftLogin<'_>> {
        self.login.as_ref()
    }

    pub async fn tick(&mut self) {
        if !matches!(self.phase, LoginPhase::WaitingForMicrosoft) {
            return;
        }

        let Some(login) = self.login.clone() else {
            return;
        };

        let interval = Duration::from_secs(login.device_code.interval.max(2));
        if self
            .last_poll
            .is_some_and(|last_poll| last_poll.elapsed() < interval)
        {
            return;
        }

        self.last_poll = Some(Instant::now());

        match login.auth.poll_device_login(&login.device_code).await {
            Ok(Some(account)) => {
                self.account = Some(account);
                self.phase = LoginPhase::Complete;
                self.error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.phase = LoginPhase::Choosing;
                self.login = None;
                self.error = Some(error.to_string());
            }
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if key.code == KeyCode::Esc {
            return true;
        }

        match self.phase {
            LoginPhase::WaitingForMicrosoft => self.handle_waiting_key(key),
            LoginPhase::Complete => self.handle_complete_key(key),
            LoginPhase::Choosing => self.handle_choosing_key(key).await,
        }

        false
    }

    pub async fn handle_click(&mut self, target: LoginClickTarget) {
        if matches!(self.phase, LoginPhase::WaitingForMicrosoft) {
            return;
        }

        match target {
            LoginClickTarget::Microsoft => {
                if matches!(self.mode, LoginMode::Microsoft)
                    && matches!(self.phase, LoginPhase::Choosing)
                {
                    self.submit().await;
                } else {
                    self.cancel_ready_account();
                    self.mode = LoginMode::Microsoft;
                    self.error = None;
                }
            }
            LoginClickTarget::Offline => {
                if matches!(self.mode, LoginMode::Offline)
                    && matches!(self.phase, LoginPhase::Choosing)
                {
                    self.submit().await;
                } else {
                    self.cancel_ready_account();
                    self.mode = LoginMode::Offline;
                    self.error = None;
                }
            }
        }
    }

    fn handle_waiting_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Backspace) {
            self.login = None;
            self.phase = LoginPhase::Choosing;
        }
    }

    fn handle_complete_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => self.cancel_ready_account(),
            KeyCode::Tab => {
                self.cancel_ready_account();
                self.toggle_mode();
            }
            KeyCode::Left => {
                self.cancel_ready_account();
                self.mode = LoginMode::Microsoft;
            }
            KeyCode::Right => {
                self.cancel_ready_account();
                self.mode = LoginMode::Offline;
            }
            _ => {}
        }
    }

    async fn handle_choosing_key(&mut self, key: KeyEvent) {
        if matches!(self.mode, LoginMode::Offline) {
            self.handle_offline_key(key).await;
        } else {
            self.handle_microsoft_key(key).await;
        }
    }

    async fn handle_offline_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.submit().await,
            KeyCode::Backspace => {
                self.username.pop();
            }
            KeyCode::Char(character) => {
                if self.username.len() < 16
                    && (character.is_ascii_alphanumeric() || character == '_')
                {
                    self.username.push(character);
                    self.error = None;
                }
            }
            KeyCode::Left | KeyCode::Tab => {
                self.mode = LoginMode::Microsoft;
                self.error = None;
            }
            _ => {}
        }
    }

    async fn handle_microsoft_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Right | KeyCode::Tab => {
                self.mode = LoginMode::Offline;
                self.error = None;
            }
            KeyCode::Enter => self.submit().await,
            _ => {}
        }
    }

    async fn submit(&mut self) {
        self.error = None;

        match self.mode {
            LoginMode::Offline => match create_offline_account(&self.username) {
                Ok(account) => {
                    self.account = Some(account);
                    self.phase = LoginPhase::Complete;
                }
                Err(error) => self.error = Some(error.to_string()),
            },
            LoginMode::Microsoft => self.start_microsoft_login().await,
        }
    }

    async fn start_microsoft_login(&mut self) {
        let client_id = env::var("CRAFTCTL_MICROSOFT_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_MICROSOFT_CLIENT_ID.to_string());

        let auth = MicrosoftAuth::create(self.client, client_id);

        match auth.start_device_login().await {
            Ok(device_code) => {
                let browser_uri = device_code.browser_uri();
                let browser_result = open(&browser_uri);
                self.login = Some(MicrosoftLogin { auth, device_code });
                self.phase = LoginPhase::WaitingForMicrosoft;
                self.last_poll = None;
                if browser_result.is_err() {
                    self.error = Some("Could not open browser. Use the link above.".to_string());
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn cancel_ready_account(&mut self) {
        self.account = None;
        self.phase = LoginPhase::Choosing;
        self.error = None;
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            LoginMode::Microsoft => LoginMode::Offline,
            LoginMode::Offline => LoginMode::Microsoft,
        };
    }
}
