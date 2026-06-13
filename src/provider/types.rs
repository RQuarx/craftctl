#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    Mod,
    Modpack,
    ResourcePack,
    ShaderPack,
    DataPack,
}

#[derive(Debug)]
pub enum ProviderError {
    RequestError(reqwest::Error),
    InvalidFormat(String),
}

#[derive(Debug)]
pub enum ProviderType {
    Modrinth,
    CurseForge,
    Ftb
}

#[derive(Debug)]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug)]
pub struct Project {
    pub provider: ProviderType,

    pub project_id: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub supported_versions: Vec<String>,
}

#[derive(Debug)]
pub struct Dependency {
    pub project_id: String,
    pub dependency_type: DependencyType,
}

pub struct Version {
    pub id: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<File>,
    pub dependencies: Vec<Dependency>
}

pub struct File {
    pub url: String,
    pub filename: String,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub size: Option<u64>,
}