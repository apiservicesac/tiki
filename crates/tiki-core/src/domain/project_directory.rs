use std::collections::HashMap;

use super::{Project, ProjectId};

/// Looks up projects by id without going back to the database.
#[derive(Debug, Clone, Default)]
pub struct ProjectDirectory {
    projects_by_id: HashMap<ProjectId, Project>,
}

impl ProjectDirectory {
    pub fn new(projects: impl IntoIterator<Item = Project>) -> Self {
        let projects_by_id = projects
            .into_iter()
            .filter_map(|project| project.id.map(|id| (id, project)))
            .collect();
        Self { projects_by_id }
    }

    pub fn find(&self, id: ProjectId) -> Option<&Project> {
        self.projects_by_id.get(&id)
    }

    /// Name of the project, if the task has one.
    pub fn name_of(&self, id: Option<ProjectId>) -> Option<&str> {
        id.and_then(|project_id| self.find(project_id))
            .map(|project| project.name.as_str())
    }

    pub fn projects(&self) -> impl Iterator<Item = &Project> {
        self.projects_by_id.values()
    }
}
