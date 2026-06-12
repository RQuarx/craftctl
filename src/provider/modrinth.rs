use serde::Deserialize;

use crate::provider::{types::ProviderType, *};

struct ModrinthSearchQuery {
    pub query: String,
    pub facets: String, // JSON string
    pub index: String,
    pub limit: u32,
    pub offset: u32,
}

impl ModrinthSearchQuery {
    pub fn from_filter(filter: &SearchFilter) -> Self {
        let mut facets: Vec<Vec<String>> = Vec::new();

        if let Some(pt) = &filter.project_type {
            facets.push(vec![format!(
                "project_type:{}",
                project_type_to_modrinth(pt)
            )]);
        }

        for version in &filter.versions {
            facets.push(vec![format!("versions:{version}")]);
        }

        for loader in &filter.loaders {
            facets.push(vec![format!("categories:{loader}")]);
        }

        Self {
            query: filter.query.clone(),
            facets: serde_json::to_string(&facets).unwrap_or_else(|_| "[]".to_string()),
            index: "relevance".into(),
            limit: 20,
            offset: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthSearchProject>,
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchProject {
    project_id: String,
    author: String,
    title: String,
    description: String,
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthProjectResponse {
    id: String,
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    id: String,

    #[serde(rename = "version_number")]
    version: String,

    game_versions: Vec<String>,
    loaders: Vec<String>,

    files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    size: u64,

    hashes: ModrinthHashes,
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha1: Option<String>,
    sha512: Option<String>,
}

pub struct Modrinth<'a> {
    client: &'a HttpClient,
}

impl<'a> Provider<'a> for Modrinth<'a> {
    fn create(client: &'a HttpClient) -> Self {
        Self { client }
    }

    async fn search(&self, filter: SearchFilter) -> Result<Vec<Project>, ProviderError> {
        let query = ModrinthSearchQuery::from_filter(&filter);

        let mut url = reqwest::Url::parse("https://api.modrinth.com/v2/search")
            .map_err(|e| ProviderError::InvalidFormat(e.to_string()))?;

        url.query_pairs_mut()
            .append_pair("query", &query.query)
            .append_pair("facets", &query.facets)
            .append_pair("index", &query.index)
            .append_pair("limit", &query.limit.to_string())
            .append_pair("offset", &query.offset.to_string());

        let response: ModrinthSearchResponse = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(ProviderError::RequestError)?
            .json()
            .await
            .map_err(ProviderError::RequestError)?;

        println!("{:#?}", response);

        Ok(response
            .hits
            .into_iter()
            .map(|project| Project {
                provider: ProviderType::Modrinth,
                project_id: project.project_id,
                author: project.author,
                title: project.title,
                description: project.description,
                supported_versions: project.versions,
            })
            .collect())
    }

    async fn get_project(&self, id: impl ToString) -> Result<Project, ProviderError> {
        let url = format!("https://api.modrinth.com/v2/project/{}", id.to_string());

        let project: ModrinthProjectResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(ProviderError::RequestError)?
            .json()
            .await
            .map_err(ProviderError::RequestError)?;

        Ok(Project {
            provider: ProviderType::Modrinth,
            project_id: project.id,
            author: String::new(),
            title: project.title,
            description: project.description,
            supported_versions: Vec::new(),
        })
    }

    async fn get_versions(&self, id: impl ToString) -> Result<Vec<Version>, ProviderError> {
        let url = format!(
            "https://api.modrinth.com/v2/project/{}/version",
            id.to_string()
        );

        let versions: Vec<ModrinthVersion> = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(ProviderError::RequestError)?
            .json()
            .await
            .map_err(ProviderError::RequestError)?;

        Ok(versions
            .into_iter()
            .map(|version| Version {
                id: version.id,
                version_number: version.version,
                game_versions: version.game_versions,
                loaders: version.loaders,
                files: version
                    .files
                    .into_iter()
                    .map(|file| File {
                        url: file.url,
                        filename: file.filename,
                        sha1: file.hashes.sha1,
                        sha512: file.hashes.sha512,
                        size: Some(file.size),
                    })
                    .collect(),
            })
            .collect())
    }
}

fn project_type_to_modrinth(t: &ProjectType) -> &'static str {
    match t {
        ProjectType::Mod => "mod",
        ProjectType::Modpack => "modpack",
        ProjectType::ResourcePack => "resourcepack",
        ProjectType::ShaderPack => "shader",
        ProjectType::DataPack => "datapack",
    }
}
