use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{error::LauncherError, result::Result};

use super::InstanceProfile;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    #[default]
    Vanilla,
    Fabric,
    Forge,
    Quilt,
    #[serde(rename = "neoforge")]
    NeoForge,
}

impl LoaderKind {
    pub const ALL: [Self; 5] = [
        Self::Vanilla,
        Self::Fabric,
        Self::Forge,
        Self::Quilt,
        Self::NeoForge,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::Quilt => "Quilt",
            Self::NeoForge => "NeoForge",
        }
    }
}

impl fmt::Display for LoaderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display_name())
    }
}

#[derive(Debug, Clone)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub minecraft_version: String,
    pub loader: LoaderKind,
    pub loader_version: String,
    pub profile: InstanceProfile,
}

impl CreateInstanceRequest {
    pub fn create(name: impl Into<String>, minecraft_version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            minecraft_version: minecraft_version.into(),
            loader: LoaderKind::default(),
            loader_version: "latest".to_string(),
            profile: InstanceProfile::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: LoaderKind,
    pub loader_version: String,
    #[serde(flatten)]
    pub profile: InstanceProfile,
}

#[derive(Debug, Deserialize)]
struct InstanceToml {
    id: String,
    name: String,
    minecraft_version: String,
    loader: LoaderKind,
    loader_version: String,
    java_runtime: Option<String>,
    memory_min_mb: Option<u16>,
    memory_max_mb: Option<u16>,
    profile: Option<InstanceProfile>,
}

impl<'de> Deserialize<'de> for Instance {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = InstanceToml::deserialize(deserializer)?;
        let profile = match serialized.profile {
            Some(profile) => profile,
            None => InstanceProfile {
                java_runtime: required(serialized.java_runtime, "java_runtime")?,
                memory_min_mb: required(serialized.memory_min_mb, "memory_min_mb")?,
                memory_max_mb: required(serialized.memory_max_mb, "memory_max_mb")?,
            },
        };

        let request = CreateInstanceRequest {
            name: serialized.name,
            minecraft_version: serialized.minecraft_version,
            loader: serialized.loader,
            loader_version: serialized.loader_version,
            profile,
        };

        Self::create(serialized.id, request).map_err(de::Error::custom)
    }
}

impl Instance {
    pub fn create(id: impl Into<String>, request: CreateInstanceRequest) -> Result<Self> {
        let id = clean_id(id.into())?;
        let name = clean_required("name", request.name)?;
        let minecraft_version = clean_required("minecraft version", request.minecraft_version)?;
        let loader_version = clean_optional(request.loader_version, "latest");

        validate_profile(&request.profile)?;

        Ok(Self {
            id,
            name,
            minecraft_version,
            loader: request.loader,
            loader_version,
            profile: request.profile,
        })
    }

    pub fn summary(&self) -> String {
        format!("{} {}", self.loader.display_name(), self.minecraft_version)
    }

    pub fn update(&mut self, request: CreateInstanceRequest) -> Result<()> {
        let name = clean_required("name", request.name)?;
        let minecraft_version = clean_required("minecraft version", request.minecraft_version)?;
        let loader_version = clean_optional(request.loader_version, "latest");

        validate_profile(&request.profile)?;

        self.name = name;
        self.minecraft_version = minecraft_version;
        self.loader = request.loader;
        self.loader_version = loader_version;
        self.profile = request.profile;

        Ok(())
    }
}

fn required<T, E>(value: Option<T>, field: &'static str) -> std::result::Result<T, E>
where
    E: de::Error,
{
    value.ok_or_else(|| E::missing_field(field))
}

pub fn instance_id_from_name(name: &str) -> String {
    let mut id = String::new();
    let mut last_was_separator = false;

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            id.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !id.is_empty() {
            id.push('-');
            last_was_separator = true;
        }
    }

    while id.ends_with('-') {
        id.pop();
    }

    if id.is_empty() {
        "instance".to_string()
    } else {
        id
    }
}

pub fn is_safe_instance_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn clean_id(id: String) -> Result<String> {
    let id = clean_required("id", id)?;

    if is_safe_instance_id(&id) {
        Ok(id)
    } else {
        Err(LauncherError::InvalidInstance(
            "id can only contain letters, numbers, hyphens, and underscores".to_string(),
        ))
    }
}

fn clean_required(field: &str, value: String) -> Result<String> {
    let value = value.trim().to_string();

    if value.is_empty() {
        Err(LauncherError::InvalidInstance(format!(
            "{field} cannot be empty"
        )))
    } else {
        Ok(value)
    }
}

fn clean_optional(value: String, fallback: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn validate_profile(profile: &InstanceProfile) -> Result<()> {
    if profile.memory_min_mb == 0 {
        return Err(LauncherError::InvalidInstance(
            "minimum memory must be greater than zero".to_string(),
        ));
    }

    if profile.memory_max_mb < profile.memory_min_mb {
        return Err(LauncherError::InvalidInstance(
            "maximum memory must be at least the minimum memory".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Instance, instance_id_from_name};

    #[test]
    fn builds_stable_instance_ids_from_names() {
        assert_eq!(instance_id_from_name("Vanilla 1.21"), "vanilla-1-21");
        assert_eq!(instance_id_from_name("  Fabric ++ Pack  "), "fabric-pack");
        assert_eq!(instance_id_from_name("!!!"), "instance");
    }

    #[test]
    fn reads_current_flat_instance_toml() {
        let instance: Instance = toml::from_str(
            r#"
id = "fabric-pack"
name = "Fabric Pack"
minecraft_version = "1.21.10"
loader = "fabric"
loader_version = "latest"
java_runtime = "system"
memory_min_mb = 1024
memory_max_mb = 4096
"#,
        )
        .unwrap();

        assert_eq!(instance.id, "fabric-pack");
        assert_eq!(instance.profile.java_runtime, "system");
    }

    #[test]
    fn reads_legacy_nested_profile_instance_toml() {
        let instance: Instance = toml::from_str(
            r#"
id = "another-cool-instance-wow"
name = "another cool instance wow!"
minecraft_version = "latest-release"
loader = "vanilla"
loader_version = "latest"

[profile]
java_runtime = "system"
memory_min_mb = 1024
memory_max_mb = 4096
"#,
        )
        .unwrap();

        assert_eq!(instance.id, "another-cool-instance-wow");
        assert_eq!(instance.profile.memory_max_mb, 4096);
    }
}
