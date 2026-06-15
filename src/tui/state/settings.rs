#[derive(Debug)]
pub struct SettingsState {
    pub config_root: String,
    status: Option<String>,
}

impl SettingsState {
    pub(crate) fn create(config_root: String) -> Self {
        Self {
            config_root,
            status: None,
        }
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub(crate) fn set_status(&mut self, status: impl Into<String>) {
        self.status = Some(status.into());
    }

    pub(crate) fn clear_status(&mut self) {
        self.status = None;
    }
}
