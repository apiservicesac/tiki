use chrono::NaiveTime;

use crate::i18n::Locale;

/// The single place where Tiki's tunable values live.
///
/// Every other module receives these values from here instead of declaring its own
/// constants, so changing a default never means hunting through files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TikiSettings {
    /// Language used for the interface, briefs and voice.
    pub locale: Locale,
    /// When the morning brief is prepared.
    pub morning_brief_time: NaiveTime,
    /// When the end-of-day close is prepared.
    pub evening_close_time: NaiveTime,
    /// A task due within this many days counts as "due soon" (0 = today, 1 = today and tomorrow).
    pub due_soon_window_days: i64,
    /// A task waiting on others for at least this many days asks for a follow-up.
    pub follow_up_after_days: i64,
    /// A meeting starting within this many minutes puts Tiki on alert.
    pub meeting_alert_minutes: i64,
    /// Maximum number of tasks the morning plan suggests for the day.
    pub daily_focus_task_count: usize,
    /// Colors given to new clients, in turn.
    pub client_colors: Vec<String>,
}

impl Default for TikiSettings {
    fn default() -> Self {
        Self {
            locale: Locale::Spanish,
            morning_brief_time: NaiveTime::from_hms_opt(8, 30, 0).expect("valid time"),
            evening_close_time: NaiveTime::from_hms_opt(18, 0, 0).expect("valid time"),
            due_soon_window_days: 2,
            follow_up_after_days: 3,
            meeting_alert_minutes: 15,
            daily_focus_task_count: 3,
            client_colors: [
                "#2F6BD8", "#17837D", "#C2410C", "#7357E8", "#B0397A", "#B45309",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}
