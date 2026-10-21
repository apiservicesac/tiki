use crate::briefing::DailyBrief;

/// Everything the resolver looks at to pick a mood.
pub struct MoodSignals<'brief> {
    pub brief: &'brief DailyBrief,
    /// Minutes until the next meeting starts, if there is one today.
    pub minutes_to_next_meeting: Option<i64>,
    pub focus_mode: bool,
}
