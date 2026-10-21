use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{ClientDirectory, Task, TaskId};
use crate::i18n::{DateFormatter, Translator};

/// One delivered task in the history, ready to draw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveredItem {
    pub task_id: TaskId,
    pub title: String,
    pub client_name: Option<String>,
    pub client_color: Option<String>,
    /// "hoy", "ayer" or a short date.
    pub delivered_label: String,
}

impl DeliveredItem {
    pub fn present(
        task: &Task,
        clients: &ClientDirectory,
        translator: &Translator,
        today: NaiveDate,
    ) -> Option<Self> {
        let client = task.client_id.and_then(|client_id| clients.find(client_id));
        Some(Self {
            task_id: task.id?,
            title: task.title.clone(),
            client_name: client.map(|client| client.name.clone()),
            client_color: client.map(|client| client.color.clone()),
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
