mod home;
mod settings;

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    auth::{LoginFlow, LoginPhase},
    config::{ConfigManager, LauncherConfig},
    http::HttpClient,
    instances::InstanceManager,
    result::Result,
};

pub use home::{CreateField, HomeMode, HomeState};
pub use settings::SettingsState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiScreen {
    Login,
    Home,
    Settings,
}

#[derive(Debug)]
pub struct TuiState {
    pub login: LoginFlow,
    pub home: HomeState,
    pub settings: SettingsState,
    pub config: LauncherConfig,
    config_manager: ConfigManager,
    screen: TuiScreen,
    pub tick: usize,
}

impl TuiState {
    pub fn create(
        client: Arc<HttpClient>,
        instance_manager: InstanceManager,
        config_manager: ConfigManager,
        config: LauncherConfig,
    ) -> Result<Self> {
        Ok(Self {
            login: LoginFlow::create(client),
            home: HomeState::create(instance_manager)?,
            settings: SettingsState::create(config_manager.root().display().to_string()),
            config,
            config_manager,
            screen: TuiScreen::Login,
            tick: usize::default(),
        })
    }

    pub fn screen(&self) -> TuiScreen {
        self.screen
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        match self.screen {
            TuiScreen::Login => self.handle_login_key(key).await,
            TuiScreen::Home => self.handle_home_key(key),
            TuiScreen::Settings => self.handle_settings_key(key),
        }
    }

    pub async fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if matches!(self.screen, TuiScreen::Login) {
            self.login.tick().await;
        }
    }

    async fn handle_login_key(&mut self, key: KeyEvent) -> Result<bool> {
        if matches!(self.login.phase(), LoginPhase::Complete) && key.code == KeyCode::Enter {
            if let Some(account) = self.login.account() {
                self.home
                    .set_status(format!("Signed in as {}", account.username));
            }

            self.screen = TuiScreen::Home;
            return Ok(false);
        }

        Ok(self.login.handle_key(key).await)
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> Result<bool> {
        if matches!(self.home.mode, HomeMode::Browsing)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.settings.clear_status();
            self.screen = TuiScreen::Settings;
            return Ok(false);
        }

        self.home.handle_key(key, self.config.ui.vim_mode)
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.screen = TuiScreen::Home;
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.config.ui.vim_mode = !self.config.ui.vim_mode;
                self.config_manager.save(&self.config)?;
                let state = if self.config.ui.vim_mode {
                    "enabled"
                } else {
                    "disabled"
                };
                self.settings.set_status(format!("Vim mode {state}"));
            }
            _ => {}
        }

        Ok(false)
    }
}
