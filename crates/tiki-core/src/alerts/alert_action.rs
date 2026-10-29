use serde::Serialize;

use super::AlertKind;

/// What the user can do with an alert; the island offers one button or link for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertAction {
    /// Opens what the alert is about: its item, the brief as a receipt, the Agenda, or the
    /// choice of what to record.
    Open,
    /// Puts the alert away: the reminder already rang and is done.
    Done,
    /// Puts the item's reminder off for a while; only an item with a reminder of its own.
    Snooze,
    /// Opens the meeting's video call.
    Join,
    /// Starts recording the call or meeting, audio only.
    RecordAudio,
    /// Asks which screen or window to record with the audio.
    RecordVideo,
}

impl AlertAction {
    /// The actions an alert of this kind offers, in the order to show them. Snoozing puts
    /// off the item's own reminder, so a follow-up nudge, a brief or a meeting cannot be
    /// snoozed; only a meeting with a video call can be joined; the offer to record starts
    /// audio or asks for the screen to record.
    pub fn offered_for(kind: AlertKind, has_link: bool) -> Vec<Self> {
        let mut actions = vec![AlertAction::Open];
        match kind {
            AlertKind::Reminder => actions.extend([AlertAction::Done, AlertAction::Snooze]),
            AlertKind::Task => actions.push(AlertAction::Snooze),
            AlertKind::Meeting if has_link => actions.push(AlertAction::Join),
            AlertKind::Recording => {
                actions.extend([AlertAction::RecordAudio, AlertAction::RecordVideo]);
            }
            AlertKind::Meeting | AlertKind::FollowUp | AlertKind::Morning | AlertKind::Evening => {}
        }
        actions
    }
}
