use std::{
    env,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::screens::login::LoginClickTarget;
use crate::{
    auth::{Account, MicrosoftAuth, MicrosoftLogin, create_offline_account},
    result::Result,
    tui::{self, LoginMode, LoginPhase},
};

const DEFAULT_MICROSOFT_CLIENT_ID: &str = "00000000402b5328";

pub async fn run() -> Result<()> {
    let mut terminal = tui::terminal::init()?;
    let should_quit = Arc::new(AtomicBool::new(false));
    install_ctrlc_handler(Arc::clone(&should_quit));
    let mut app = App::new();
    let result = tui::events::run(&mut terminal, &mut app, should_quit).await;
    tui::terminal::restore(&mut terminal)?;
    result
}

#[derive(Debug)]
pub struct App {
    pub mode: LoginMode,
    pub phase: LoginPhase,
    pub username: String,
    pub account: Option<Account>,
    pub error: Option<String>,
    pub tick: usize,
    login: Option<MicrosoftLogin>,
    last_poll: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            mode: LoginMode::Microsoft,
            phase: LoginPhase::Choosing,
            username: String::new(),
            account: None,
            error: None,
            tick: 0,
            login: None,
            last_poll: None,
        }
    }

    pub async fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);

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

        if matches!(self.phase, LoginPhase::WaitingForMicrosoft) {
            if matches!(key.code, KeyCode::Backspace) {
                self.login = None;
                self.phase = LoginPhase::Choosing;
            }
            return false;
        }

        if matches!(self.phase, LoginPhase::Complete) {
            match key.code {
                KeyCode::Backspace => self.cancel_ready_account(),
                KeyCode::Tab => {
                    self.cancel_ready_account();
                    self.mode = match self.mode {
                        LoginMode::Microsoft => LoginMode::Offline,
                        LoginMode::Offline => LoginMode::Microsoft,
                    };
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

            return false;
        }

        if matches!(self.mode, LoginMode::Offline) {
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
                KeyCode::Left => {
                    self.mode = LoginMode::Microsoft;
                    self.error = None;
                }
                KeyCode::Tab => {
                    self.mode = LoginMode::Microsoft;
                    self.error = None;
                }
                _ => {}
            }

            return false;
        }

        match key.code {
            KeyCode::Right | KeyCode::Tab => {
                self.mode = LoginMode::Offline;
                self.error = None;
            }
            KeyCode::Enter => self.submit().await,
            _ => {}
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

        let auth = MicrosoftAuth::new(client_id);

        match auth.start_device_login().await {
            Ok(device_code) => {
                let browser_uri = device_code.browser_uri();
                let browser_result = open_browser(&browser_uri);
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

    pub fn microsoft_login(&self) -> Option<&MicrosoftLogin> {
        self.login.as_ref()
    }
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

impl App {
    fn cancel_ready_account(&mut self) {
        self.account = None;
        self.phase = LoginPhase::Choosing;
        self.error = None;
    }
}

fn install_ctrlc_handler(should_quit: Arc<AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        should_quit.store(true, Ordering::SeqCst);
    });
}
