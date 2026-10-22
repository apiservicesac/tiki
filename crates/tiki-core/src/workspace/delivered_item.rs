use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{ProjectDirectory, Task, TaskId};
use crate::i18n::{DateFormatter, Translator};

/// One delivered task in the history, ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveredItem {
    pub task_id: TaskId,
    pub title: String,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    /// "hoy", "ayer" or a short date.
    pub delivered_label: String,
}

impl DeliveredItem {
    pub fn present(
        task: &Task,
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
            project_name: project.map(|project| project.name.clone()),
            project_color: project.map(|project| project.color.clone()),
            delivered_label: Self::label(task.completed_on, translator, today),
        })
    }

    fn label(day: Option<NaiveDate>, translator: &Translator, today: NaiveDate) -> String {
        match day.map(|day| (today - day).num_days()) {
            Some(0) => translator.text("history.today"),
            Some(1) => translator.text("history.yesterday"),
            Some(_) => {
                day.map_or_else(String::new, |day| DateFormatter::new(translator).short(day))
            }
            None => String::new(),
        }
    }
}
