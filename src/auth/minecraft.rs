use crate::{auth::Account, error::LauncherError, result::Result};

pub fn create_offline_account(username: &str) -> Result<Account> {
    let username = username.trim();

    if !(3..=16).contains(&username.len()) {
        return Err(LauncherError::Auth(
            "offline usernames must be 3 to 16 characters".to_string(),
        ));
    }

    if !username
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(LauncherError::Auth(
            "offline usernames can only use letters, numbers, and underscores".to_string(),
        ));
    }

    Ok(Account::offline(username))
}
