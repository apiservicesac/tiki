use chrono::NaiveDate;

use super::{ProjectChange, ProjectSummary};
use crate::domain::{Project, ProjectId, Task};
use crate::error::{TikiError, TikiResult};
use crate::settings::TikiSettings;
use crate::storage::Database;

/// Creates, changes and removes projects, and lists them with their workload.
pub struct ProjectBook<'context> {
    database: &'context Database,
    settings: &'context TikiSettings,
}

impl<'context> ProjectBook<'context> {
    pub fn new(database: &'context Database, settings: &'context TikiSettings) -> Self {
        Self { database, settings }
    }

    /// Every project, busiest first: pressing work, then open work, then by name.
    pub fn summaries(
        &self,
        open_tasks: &[Task],
        today: NaiveDate,
    ) -> TikiResult<Vec<ProjectSummary>> {
        let mut summaries: Vec<ProjectSummary> = self
            .database
            .projects()
            .list()?
            .into_iter()
            .filter_map(|project| {
                let project_id = project.id?;
                let tasks = open_tasks
                    .iter()
                    .filter(|task| task.project_id == Some(project_id) && task.status.is_open());
                let (open_count, pressing_count) = tasks.fold((0, 0), |(open, pressing), task| {
                    let is_pressing = task.is_overdue(today)
                        || task.is_due_within(today, self.settings.due_soon_window_days);
                    (open + 1, pressing + usize::from(is_pressing))
                });
                Some(ProjectSummary {
                    project_id,
                    name: project.name,
                    color: project.color,
                    open_count,
                    pressing_count,
                })
            })
            .collect();
        summaries.sort_by(|first, second| {
            second
                .pressing_count
                .cmp(&first.pressing_count)
                .then(second.open_count.cmp(&first.open_count))
                .then_with(|| first.name.to_lowercase().cmp(&second.name.to_lowercase()))
        });
        Ok(summaries)
    }

    /// Saves a new project with the next color of the palette.
    pub fn create(&self, name: &str) -> TikiResult<ProjectId> {
        let name = self.available_name(name, None)?;
        let existing_projects = self.database.projects().list()?.len();
        let palette = &self.settings.project_colors;
        let color = palette
            .get(existing_projects % palette.len().max(1))
            .cloned()
            .unwrap_or_default();
        self.database.projects().insert(&Project::new(name, color))
    }

    pub fn change(&self, project_id: ProjectId, change: ProjectChange) -> TikiResult<()> {
        let name = self.available_name(&change.name, Some(project_id))?;
        self.database.projects().update(&Project {
            id: Some(project_id),
            name,
            color: change.color,
        })
    }

    /// Removes a project; their tasks stay, without project.
    pub fn remove(&self, project_id: ProjectId) -> TikiResult<()> {
        self.database.projects().delete(project_id)
    }

    /// The trimmed name, if no other project already uses it (ignoring case).
    fn available_name(&self, name: &str, project_id: Option<ProjectId>) -> TikiResult<String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TikiError::EmptyProjectName);
        }
        let taken = self
            .database
            .projects()
            .find_by_name(name)?
            .is_some_and(|existing| existing.id != project_id);
        if taken {
            return Err(TikiError::DuplicateProjectName(name.to_owned()));
        }
        Ok(name.to_owned())
    }
}
