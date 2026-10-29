use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{ProjectDirectory, ProjectId, Task, TaskId};
use crate::i18n::{DateFormatter, Translator};

/// One delivered task in the history, ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveredItem {
    pub task_id: TaskId,
    pub title: String,
    pub project_id: Option<ProjectId>,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    /// "hoy", "ayer" or a short date.
    pub delivered_label: String,
}

impl DeliveredItem {
    /// The task as delivered on `delivered_on`; `None` for a task never saved.
    pub fn present(
        task: &Task,
        delivered_on: NaiveDate,
        projects: &ProjectDirectory,
        translator: &Translator,
        today: NaiveDate,
    ) -> Option<Self> {
        let project = task
            .project_id
            .and_then(|project_id| projects.find(project_id));
        Some(Self {
            task_id: task.id?,
            title: task.title.clone(),
            project_id: task.project_id,
            project_name: project.map(|project| project.name.clone()),
            project_color: project.map(|project| project.color.clone()),
            delivered_label: Self::label(delivered_on, translator, today),
        })
    }

    fn label(day: NaiveDate, translator: &Translator, today: NaiveDate) -> String {
        match (today - day).num_days() {
            0 => translator.text("history.today"),
            1 => translator.text("history.yesterday"),
            _ => DateFormatter::new(translator).short(day),
        }
    }
}
