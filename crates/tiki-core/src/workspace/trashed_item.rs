use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{ItemKind, ProjectDirectory, Task, TaskId, TrashPolicy};
use crate::i18n::Translator;

/// One item in the trash, ready to draw: when it went there and when it is deleted forever.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashedItem {
    pub task_id: TaskId,
    pub kind: ItemKind,
    pub title: String,
    pub project_name: Option<String>,
    pub project_color: Option<String>,
    /// "hoy", "ayer" or "hace 3 días".
    pub trashed_label: String,
    /// "se borra hoy", "se borra mañana" or "se borra en 27 días".
    pub deletes_in_label: String,
}

impl TrashedItem {
    /// The item as the trash shows it on `today`; `None` for an item never saved or not in
    /// the trash.
    pub fn present(
        task: &Task,
        projects: &ProjectDirectory,
        policy: &TrashPolicy,
        translator: &Translator,
        today: NaiveDate,
    ) -> Option<Self> {
        let trashed_at = task.trashed_at?;
        let project = task
            .project_id
            .and_then(|project_id| projects.find(project_id));
        let days_in_trash = (today - trashed_at.date()).num_days();
        let days_left = (policy.deletes_on(trashed_at) - today).num_days().max(0);
        Some(Self {
            task_id: task.id?,
            kind: task.kind,
            title: task.title.clone(),
            project_name: project.map(|project| project.name.clone()),
            project_color: project.map(|project| project.color.clone()),
            trashed_label: Self::trashed_label(days_in_trash, translator),
            deletes_in_label: Self::deletes_in_label(days_left, translator),
        })
    }

    fn trashed_label(days_in_trash: i64, translator: &Translator) -> String {
        match days_in_trash {
            0 => translator.text("trash.trashed_today"),
            1 => translator.text("trash.trashed_yesterday"),
            days => translator.format("trash.trashed_days_ago", &[("days", &days.to_string())]),
        }
    }

    fn deletes_in_label(days_left: i64, translator: &Translator) -> String {
        match days_left {
            0 => translator.text("trash.deletes_today"),
            1 => translator.text("trash.deletes_tomorrow"),
            days => translator.format("trash.deletes_in_days", &[("days", &days.to_string())]),
        }
    }
}
