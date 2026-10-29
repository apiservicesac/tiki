use super::{BriefDetail, BriefItem, DailyBrief};
use crate::domain::Priority;

/// Why an item is the one to do now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusReason {
    Overdue,
    DueToday,
    /// Urgent or high-priority work on the user.
    Important,
    DueSoon,
    FollowUp,
}

impl FocusReason {
    /// Stable key, used in translation keys (`focus.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            FocusReason::Overdue => "overdue",
            FocusReason::DueToday => "due_today",
            FocusReason::Important => "important",
            FocusReason::DueSoon => "due_soon",
            FocusReason::FollowUp => "follow_up",
        }
    }
}

/// The one item to do now, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusPick<'brief> {
    pub item: &'brief BriefItem,
    pub reason: FocusReason,
}

/// Answers "what do I do now?" from the brief: the most important overdue item, else what
/// is due today, else urgent or important work, else what is due soon, else a follow-up.
pub struct FocusPicker;

impl FocusPicker {
    /// Work at this priority or above counts as important.
    const IMPORTANT_PRIORITY: Priority = Priority::High;

    pub fn pick(brief: &DailyBrief) -> Option<FocusPick<'_>> {
        let due_today = |item: &&BriefItem| item.detail == BriefDetail::DueIn { days: 0 };
        let important = |item: &&BriefItem| item.priority >= Self::IMPORTANT_PRIORITY;
        let candidates = [
            (brief.overdue.first(), FocusReason::Overdue),
            (brief.due_soon.iter().find(due_today), FocusReason::DueToday),
            (
                brief.your_turn.iter().find(important),
                FocusReason::Important,
            ),
            (brief.due_soon.first(), FocusReason::DueSoon),
            (brief.needs_follow_up.first(), FocusReason::FollowUp),
        ];
        candidates
            .into_iter()
            .find_map(|(item, reason)| item.map(|item| FocusPick { item, reason }))
    }
}
