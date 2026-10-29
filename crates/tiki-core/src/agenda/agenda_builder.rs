use chrono::{NaiveDate, NaiveDateTime};

use super::{AgendaDay, AgendaEntry};
use crate::calendar::MeetingItem;
use crate::domain::ItemKind;
use crate::i18n::{DateFormatter, Translator};
use crate::workspace::SnapshotItem;

/// Puts the items that ring at a time and the calendar's meetings in one list, by day and
/// by time, with the day titles and the times already written in the user's language.
pub struct AgendaBuilder<'translator> {
    translator: &'translator Translator,
}

impl<'translator> AgendaBuilder<'translator> {
    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    /// The open items with a moment to ring and every meeting, grouped by day from `today`.
    pub fn build(
        &self,
        items: &[SnapshotItem],
        meetings: &[MeetingItem],
        today: NaiveDate,
    ) -> Vec<AgendaDay> {
        let timed_items = items
            .iter()
            .filter_map(|item| item.at.map(|at| (at, Self::item_entry(item, at))));
        let calendar = meetings
            .iter()
            .map(|meeting| (meeting.starts_at, self.meeting_entry(meeting)));
        let mut entries: Vec<(NaiveDateTime, AgendaEntry)> = timed_items.chain(calendar).collect();
        // The sort is stable: at the same moment, items stay before meetings.
        entries.sort_by_key(|(at, _)| *at);

        let mut days: Vec<AgendaDay> = Vec::new();
        for (at, entry) in entries {
            match days.last_mut() {
                Some(day) if day.day == at.date() => day.entries.push(entry),
                _ => days.push(AgendaDay {
                    day: at.date(),
                    label: self.day_label(at.date(), today),
                    entries: vec![entry],
                }),
            }
        }
        days
    }

    fn item_entry(item: &SnapshotItem, at: NaiveDateTime) -> AgendaEntry {
        let time_label = DateFormatter::time(at.time());
        let item = item.clone();
        match item.kind {
            ItemKind::Reminder => AgendaEntry::Reminder { time_label, item },
            ItemKind::Task => AgendaEntry::Task { time_label, item },
        }
    }

    fn meeting_entry(&self, meeting: &MeetingItem) -> AgendaEntry {
        let time_label = if meeting.all_day {
            self.translator.text("meeting.all_day")
        } else {
            DateFormatter::new(self.translator)
                .time_range(meeting.starts_at.time(), meeting.ends_at.time())
        };
        AgendaEntry::Meeting {
            time_label,
            meeting: meeting.clone(),
        }
    }

    /// "Hoy", "Mañana" or the long date.
    fn day_label(&self, day: NaiveDate, today: NaiveDate) -> String {
        match (day - today).num_days() {
            0 => self.translator.text("agenda.today"),
            1 => self.translator.text("agenda.tomorrow"),
            _ => DateFormatter::new(self.translator).long(day),
        }
    }
}
