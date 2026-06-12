use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountKind {
    Microsoft,
    Offline,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub kind: AccountKind,
    pub access_token: Option<String>,
    pub expires_at: Option<SystemTime>,
}

impl Account {
    pub fn offline(username: impl Into<String>) -> Self {
        let username = username.into();

        Self {
            id: format!("offline:{username}"),
            username,
            kind: AccountKind::Offline,
            access_token: None,
            expires_at: None,
        }
    }

    pub fn microsoft(
        id: impl Into<String>,
        username: impl Into<String>,
        access_token: impl Into<String>,
        expires_at: SystemTime,
    ) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            kind: AccountKind::Microsoft,
            access_token: Some(access_token.into()),
            expires_at: Some(expires_at),
        }
    }
}
