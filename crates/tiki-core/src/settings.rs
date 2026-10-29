use chrono::NaiveTime;

use crate::i18n::Locale;
use crate::preferences::QuietHours;

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
    pub evening_brief_time: NaiveTime,
    /// The night a new user starts with: Tiki sleeps and alerts make no sound.
    pub quiet_hours: QuietHours,
    /// A task due within this many days counts as "due soon" (0 = today, 1 = today and tomorrow).
    pub due_soon_window_days: i64,
    /// A follow-up reminder rings at this time of the day it is due.
    pub follow_up_time: NaiveTime,
    /// A task waiting on others for at least this many days asks for a follow-up.
    pub follow_up_after_days: i64,
    /// While it still waits, the follow-up nudge comes back every this many days.
    pub follow_up_repeat_days: i64,
    /// The Agenda and the calendar copy cover today and this many days ahead.
    pub agenda_days: u64,
    /// A meeting starting within this many minutes puts Tiki on alert.
    pub meeting_alert_minutes: i64,
    /// Maximum number of tasks the morning plan suggests for the day.
    pub daily_focus_task_count: usize,
    /// How many of the user's corrections are kept and shown to the AI as examples.
    pub capture_lesson_limit: usize,
    /// How many delivered tasks the history shows, most recent first.
    pub delivered_history_limit: usize,
    /// Items stay this many days in the trash, then Tiki deletes them forever.
    pub trash_retention_days: u64,
    /// Colors given to new projects, in turn.
    pub project_colors: Vec<String>,
}

impl Default for TikiSettings {
    fn default() -> Self {
        Self {
            locale: Locale::Spanish,
            morning_brief_time: NaiveTime::from_hms_opt(8, 30, 0).expect("valid time"),
            evening_brief_time: NaiveTime::from_hms_opt(18, 0, 0).expect("valid time"),
            quiet_hours: QuietHours {
                enabled: true,
                from: NaiveTime::from_hms_opt(22, 0, 0).expect("valid time"),
                until: NaiveTime::from_hms_opt(7, 0, 0).expect("valid time"),
            },
            due_soon_window_days: 2,
            follow_up_time: NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"),
            follow_up_after_days: 3,
            follow_up_repeat_days: 2,
            meeting_alert_minutes: 15,
            agenda_days: 8,
            daily_focus_task_count: 3,
            delivered_history_limit: 300,
            trash_retention_days: 30,
            capture_lesson_limit: 20,
            // Sixteen colors that read on dark and light, so even many projects repeat little.
            project_colors: [
                "#2F6BD8", "#17837D", "#C2410C", "#7357E8", "#B0397A", "#B45309", "#0E7490",
                "#4D7C0F", "#BE123C", "#6D28D9", "#0369A1", "#A16207", "#15803D", "#9333EA",
                "#DB2777", "#475569",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}
