use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    instances::{CreateInstanceRequest, Instance, InstanceManager, LoaderKind},
    result::Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeMode {
    Browsing,
    Creating,
    Editing,
    ConfirmingDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateField {
    Name,
    MinecraftVersion,
    Loader,
    LoaderVersion,
}

impl CreateField {
    const ORDER: [Self; 4] = [
        Self::Name,
        Self::MinecraftVersion,
        Self::Loader,
        Self::LoaderVersion,
    ];

    fn next(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|field| *field == self)
            .unwrap_or_default();

        Self::ORDER[(index + 1) % Self::ORDER.len()]
    }

    fn previous(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|field| *field == self)
            .unwrap_or_default();

        Self::ORDER[(index + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
}

#[derive(Debug, Clone)]
pub struct InstanceForm {
    pub name: String,
    pub minecraft_version: String,
    pub loader: LoaderKind,
    pub loader_version: String,
    pub field: CreateField,
}

impl InstanceForm {
    fn create() -> Self {
        Self {
            name: String::new(),
            minecraft_version: "latest-release".to_string(),
            loader: LoaderKind::default(),
            loader_version: "latest".to_string(),
            field: CreateField::Name,
        }
    }

    fn from_instance(instance: &Instance) -> Self {
        Self {
            name: instance.name.clone(),
            minecraft_version: instance.minecraft_version.clone(),
            loader: instance.loader,
            loader_version: instance.loader_version.clone(),
            field: CreateField::Name,
        }
    }

    fn request(&self) -> CreateInstanceRequest {
        let mut request =
            CreateInstanceRequest::create(self.name.clone(), self.minecraft_version.clone());

        request.loader = self.loader;
        request.loader_version = self.loader_version.clone();
        request
    }

    fn next_field(&mut self) {
        self.field = self.field.next();
    }

    fn previous_field(&mut self) {
        self.field = self.field.previous();
    }

    fn next_loader(&mut self) {
        let index = loader_index(self.loader);
        self.loader = LoaderKind::ALL[(index + 1) % LoaderKind::ALL.len()];
    }

    fn previous_loader(&mut self) {
        let index = loader_index(self.loader);
        self.loader = LoaderKind::ALL[(index + LoaderKind::ALL.len() - 1) % LoaderKind::ALL.len()];
    }

    fn push(&mut self, character: char) {
        match self.field {
            CreateField::Name => {
                if !character.is_control() && self.name.chars().count() < 48 {
                    self.name.push(character);
                }
            }
            CreateField::MinecraftVersion => {
                push_version_character(&mut self.minecraft_version, character);
            }
            CreateField::Loader => {
                if character == ' ' {
                    self.next_loader();
                }
            }
            CreateField::LoaderVersion => {
                push_version_character(&mut self.loader_version, character);
            }
        }
    }

    fn pop(&mut self) {
        match self.field {
            CreateField::Name => {
                self.name.pop();
            }
            CreateField::MinecraftVersion => {
                self.minecraft_version.pop();
            }
            CreateField::Loader => {}
            CreateField::LoaderVersion => {
                self.loader_version.pop();
            }
        }
    }
}

#[derive(Debug)]
pub struct HomeState {
    manager: InstanceManager,
    pub instances: Vec<Instance>,
    pub selected: usize,
    pub mode: HomeMode,
    pub instance_form: InstanceForm,
    pub pending_delete: Option<Instance>,
    status: Option<String>,
    error: Option<String>,
}

impl HomeState {
    pub(crate) fn create(manager: InstanceManager) -> Result<Self> {
        let instances = manager.load_instances()?;

        Ok(Self {
            manager,
            instances,
            selected: 0,
            mode: HomeMode::Browsing,
            instance_form: InstanceForm::create(),
            pending_delete: None,
            status: None,
            error: None,
        })
    }

    pub fn selected_instance(&self) -> Option<&Instance> {
        self.instances.get(self.selected)
    }

    pub fn status(&self) -> Option<&str> {
        self.error.as_deref().or(self.status.as_deref())
    }

    pub fn status_is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn data_root(&self) -> &std::path::Path {
        self.manager.storage().root()
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = Some(status.into());
        self.error = None;
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent, vim_mode: bool) -> Result<bool> {
        match self.mode {
            HomeMode::Browsing => self.handle_browsing_key(key, vim_mode),
            HomeMode::Creating | HomeMode::Editing => self.handle_instance_form_key(key),
            HomeMode::ConfirmingDelete => self.handle_delete_confirmation_key(key),
        }
    }

    fn handle_browsing_key(&mut self, key: KeyEvent, vim_mode: bool) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            KeyCode::Char('j') if vim_mode => self.select_next(),
            KeyCode::Char('k') if vim_mode => self.select_previous(),
            KeyCode::Char('n') | KeyCode::Char('N') => self.start_create(),
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh()?;
                self.set_status("Instance list refreshed");
            }
            KeyCode::Char('e') | KeyCode::Char('E') => self.start_edit(),
            KeyCode::Char('d') | KeyCode::Char('D') => self.start_delete(),
            KeyCode::Enter => self.launch_selected_or_create(),
            KeyCode::Char('l') if vim_mode => self.launch_selected_or_create(),
            _ => {}
        }

        Ok(false)
    }

    fn handle_instance_form_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = HomeMode::Browsing;
                self.status = None;
                self.error = None;
            }
            KeyCode::Tab | KeyCode::Down => self.instance_form.next_field(),
            KeyCode::BackTab | KeyCode::Up => self.instance_form.previous_field(),
            KeyCode::Left if self.instance_form.field == CreateField::Loader => {
                self.instance_form.previous_loader();
            }
            KeyCode::Right if self.instance_form.field == CreateField::Loader => {
                self.instance_form.next_loader();
            }
            KeyCode::Enter => {
                if self.instance_form.field == CreateField::LoaderVersion {
                    self.submit_instance_form()?;
                } else {
                    self.instance_form.next_field();
                }
            }
            KeyCode::Backspace => {
                self.instance_form.pop();
                self.error = None;
            }
            KeyCode::Char(character) => {
                self.instance_form.push(character);
                self.error = None;
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_delete_confirmation_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => self.cancel_modal(),
            KeyCode::Char('y') | KeyCode::Char('Y') => self.submit_delete()?,
            _ => {}
        }

        Ok(false)
    }

    fn start_create(&mut self) {
        self.mode = HomeMode::Creating;
        self.instance_form = InstanceForm::create();
        self.status = None;
        self.error = None;
    }

    fn start_edit(&mut self) {
        let Some(instance) = self.selected_instance().cloned() else {
            self.error = Some("Create an instance before editing.".to_string());
            self.status = None;
            return;
        };

        self.instance_form = InstanceForm::from_instance(&instance);
        self.mode = HomeMode::Editing;
        self.status = None;
        self.error = None;
    }

    fn start_delete(&mut self) {
        let Some(instance) = self.selected_instance().cloned() else {
            self.error = Some("Create an instance before deleting.".to_string());
            self.status = None;
            return;
        };

        self.pending_delete = Some(instance);
        self.mode = HomeMode::ConfirmingDelete;
        self.status = None;
        self.error = None;
    }

    fn launch_selected_or_create(&mut self) {
        if let Some(instance) = self.selected_instance() {
            self.set_status(format!("Launch not wired yet for {}", instance.name));
        } else {
            self.start_create();
        }
    }

    fn cancel_modal(&mut self) {
        self.mode = HomeMode::Browsing;
        self.pending_delete = None;
        self.status = None;
        self.error = None;
    }

    fn submit_instance_form(&mut self) -> Result<()> {
        match self.mode {
            HomeMode::Creating => self.submit_create_form(),
            HomeMode::Editing => self.submit_edit_form(),
            _ => Ok(()),
        }
    }

    fn submit_create_form(&mut self) -> Result<()> {
        match self.manager.create_instance(self.instance_form.request()) {
            Ok(instance) => self.finish_instance_change(instance, "Created"),
            Err(error) => self.error = Some(error.to_string()),
        }

        Ok(())
    }

    fn submit_edit_form(&mut self) -> Result<()> {
        let Some(instance) = self.selected_instance().cloned() else {
            self.error = Some("No instance selected.".to_string());
            return Ok(());
        };

        match self
            .manager
            .update_instance(instance, self.instance_form.request())
        {
            Ok(instance) => self.finish_instance_change(instance, "Updated"),
            Err(error) => self.error = Some(error.to_string()),
        }

        Ok(())
    }

    fn submit_delete(&mut self) -> Result<()> {
        let Some(instance) = self.pending_delete.take() else {
            self.error = Some("No instance selected.".to_string());
            return Ok(());
        };

        match self.manager.delete_instance(&instance) {
            Ok(()) => {
                let name = instance.name;

                self.refresh()?;
                self.mode = HomeMode::Browsing;
                self.set_status(format!("Deleted {name}"));
            }
            Err(error) => {
                self.pending_delete = Some(instance);
                self.error = Some(error.to_string());
            }
        }

        Ok(())
    }

    fn finish_instance_change(&mut self, instance: Instance, action: &str) {
        let id = instance.id.clone();
        let name = instance.name.clone();

        match self.refresh() {
            Ok(()) => {
                self.select_id(&id);
                self.mode = HomeMode::Browsing;
                self.pending_delete = None;
                self.set_status(format!("{action} {name}"));
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn refresh(&mut self) -> Result<()> {
        self.instances = self.manager.load_instances()?;

        if self.instances.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.instances.len() {
            self.selected = self.instances.len() - 1;
        }

        Ok(())
    }

    fn select_id(&mut self, id: &str) {
        if let Some(index) = self.instances.iter().position(|instance| instance.id == id) {
            self.selected = index;
        }
    }

    fn select_next(&mut self) {
        if self.instances.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.instances.len();
    }

    fn select_previous(&mut self) {
        if self.instances.is_empty() {
            return;
        }

        self.selected = (self.selected + self.instances.len() - 1) % self.instances.len();
    }
}

fn loader_index(loader: LoaderKind) -> usize {
    LoaderKind::ALL
        .iter()
        .position(|candidate| *candidate == loader)
        .unwrap_or_default()
}

fn push_version_character(value: &mut String, character: char) {
    if value.chars().count() >= 32 {
        return;
    }

    if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | '+') {
        value.push(character);
    }
}
