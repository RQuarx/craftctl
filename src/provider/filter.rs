use crate::provider::types::ProjectType;

pub struct SearchFilter {
    pub query: String,
    pub project_type: Option<ProjectType>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
}

impl SearchFilter {
    pub fn build(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            project_type: None,
            versions: Vec::new(),
            loaders: Vec::new(),
        }
    }

    pub fn project_type(mut self, t: ProjectType) -> Self {
        self.project_type = Some(t);
        self
    }

    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.versions.push(v.into());
        self
    }

    pub fn loader(mut self, l: impl Into<String>) -> Self {
        self.loaders.push(l.into());
        self
    }
}
