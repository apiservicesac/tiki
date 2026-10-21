use chrono::{Days, NaiveDate};

use super::BriefDetail;
use crate::i18n::{DateFormatter, Translator};

/// Writes the short label shown next to each brief line: "mañana", "3 días", "En prueba".
pub struct BriefDetailFormatter<'translator> {
    translator: &'translator Translator,
}

impl<'translator> BriefDetailFormatter<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    pub fn label(&self, detail: BriefDetail, today: NaiveDate) -> String {
        match detail {
            BriefDetail::DueIn { days: 0 } => self.translator.text("brief.detail.due_today"),
            BriefDetail::DueIn { days: 1 } => self.translator.text("brief.detail.due_tomorrow"),
            BriefDetail::DueIn { days } => self.due_date_label(today, days),
            BriefDetail::Overdue { days: 1 } => {
                self.translator.text("brief.detail.overdue_one_day")
            }
            BriefDetail::Overdue { days } => self
                .translator
                .format("brief.detail.overdue_days", &[("days", &days.to_string())]),
            BriefDetail::Waiting { days: 1 } => {
                self.translator.text("brief.detail.waiting_one_day")
            }
            BriefDetail::Waiting { days } => self
                .translator
                .format("brief.detail.waiting_days", &[("days", &days.to_string())]),
            BriefDetail::InStatus { status } => {
                self.translator.text(&format!("status.{}", status.key()))
            }
        }
    }

    fn due_date_label(&self, today: NaiveDate, days: i64) -> String {
        let offset = Days::new(days.unsigned_abs());
        let due_date = if days >= 0 {
            today.checked_add_days(offset)
        } else {
            today.checked_sub_days(offset)
        };
        due_date.map_or_else(String::new, |date| {
            DateFormatter::new(self.translator).short(date)
        })
    }
}
