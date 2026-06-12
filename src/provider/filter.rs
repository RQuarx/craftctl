use std::collections::HashSet;

use crate::provider::types::ProjectType;

pub struct SearchFilter {
    pub query: String,
    pub project_type: Option<ProjectType>,
    pub versions: HashSet<String>,
    pub loaders: HashSet<String>,
}

impl SearchFilter {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            project_type: None,
            versions: HashSet::new(),
            loaders: HashSet::new(),
        }
    }

    pub fn project_type(mut self, t: ProjectType) -> Self {
        self.project_type = Some(t);
        self
    }

    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.versions.insert(v.into());
        self
    }

    pub fn loader(mut self, l: impl Into<String>) -> Self {
        self.loaders.insert(l.into());
        self
    }
}
