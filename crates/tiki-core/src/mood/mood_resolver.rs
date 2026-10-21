use super::{Mood, MoodSignals};
use crate::settings::TikiSettings;

/// Picks the resting mood from the state of the day.
///
/// "Curious" and "Happy" are reactions to events (a new ticket, a delivery), so the
/// interface shows them for a moment and then comes back to the mood resolved here.
pub struct MoodResolver<'settings> {
    settings: &'settings TikiSettings,
}

impl<'settings> MoodResolver<'settings> {
    pub fn new(settings: &'settings TikiSettings) -> Self {
        Self { settings }
    }

    /// The most urgent signal wins, in this order: focus, meeting, overdue, due soon, follow-up.
    pub fn resolve(&self, signals: &MoodSignals<'_>) -> Mood {
        if signals.focus_mode {
            Mood::Sleep
        } else if self.meeting_is_close(signals.minutes_to_next_meeting) {
            Mood::Alert
        } else if !signals.brief.overdue.is_empty() {
            Mood::Worried
        } else if !signals.brief.due_soon.is_empty() {
            Mood::Hurry
        } else if !signals.brief.needs_follow_up.is_empty() {
            Mood::Impatient
        } else {
            Mood::Calm
        }
    }

    fn meeting_is_close(&self, minutes_to_next_meeting: Option<i64>) -> bool {
        minutes_to_next_meeting
            .is_some_and(|minutes| (0..=self.settings.meeting_alert_minutes).contains(&minutes))
    }
}
