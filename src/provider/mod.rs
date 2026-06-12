mod types;
pub use types::ProviderError;
pub use types::Version;
pub use types::File;
pub use types::Project;
pub use types::ProjectType;

mod filter;
pub use filter::SearchFilter;
use crate::http::HttpClient;

pub trait Provider<'a> {
    fn create(client: &'a HttpClient) -> Self;
    async fn search(&self, filter: SearchFilter) -> Result<Vec<Project>, ProviderError>;
    async fn get_project(&self, id: impl ToString) -> Result<Project, ProviderError>;
    async fn get_versions(&self, id: impl ToString) -> Result<Vec<Version>, ProviderError>;
}

mod modrinth;
pub use modrinth::Modrinth;

