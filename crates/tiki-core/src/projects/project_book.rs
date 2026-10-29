use chrono::{NaiveDate, NaiveDateTime};

use super::{ProjectChange, ProjectSummary};
use crate::activity::ItemEvent;
use crate::domain::{Project, ProjectDirectory, ProjectId, Task};
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

    /// Every project, busiest first: pressing work, then open work, then by name. Standalone
    /// reminders are not work, as in the brief.
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
                let tasks = open_tasks.iter().filter(|task| {
                    task.project_id == Some(project_id) && task.counts_as_open_work()
                });
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
                    notes: project.notes,
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

    /// Every project, to look them up by id.
    pub fn directory(&self) -> TikiResult<ProjectDirectory> {
        Ok(ProjectDirectory::new(self.database.projects().list()?))
    }

    /// The project with this name, created when it does not exist yet; `None` for no name.
    /// Tiki never asks the user to keep a list of projects.
    pub fn find_or_create(&self, name: Option<&str>) -> TikiResult<Option<ProjectId>> {
        let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
            return Ok(None);
        };
        match self.database.projects().find_by_name(name)? {
            Some(project) => Ok(project.id),
            None => self.create(name).map(Some),
        }
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
            notes: change.notes.trim().to_owned(),
        })
    }

    /// Removes a project and, in the same transaction, sends its items (open and delivered)
    /// to the trash at `now`, where they stay without project until restored or deleted.
    pub fn remove(&self, project_id: ProjectId, now: NaiveDateTime) -> TikiResult<()> {
        self.database.transaction(|repositories| {
            for mut task in repositories.tasks().list_untrashed_of_project(project_id)? {
                let task_id = task.id.ok_or(TikiError::UnsavedTask)?;
                let events: Vec<ItemEvent> = task.trash(now).into_iter().collect();
                repositories.tasks().update(&task, now)?;
                repositories.activity().record(task_id, now, &events)?;
            }
            repositories.projects().delete(project_id)
        })
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
