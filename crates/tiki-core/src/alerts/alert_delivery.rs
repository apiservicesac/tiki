use chrono::NaiveTime;
use serde::Serialize;

use crate::preferences::UserPreferences;

/// How an alert reaches the user besides the island, which always shows it: with the
/// island's chime and as a Windows notification, each when the user wants it. Both keep
/// quiet during the night's quiet hours and while the user is in a call: nothing
/// interrupts, and nothing is lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertDelivery {
    pub sound: bool,
    pub windows_notification: bool,
}

impl AlertDelivery {
    pub fn decide(preferences: &UserPreferences, now: NaiveTime, in_call: bool) -> Self {
        let may_interrupt = !in_call && !preferences.quiet_hours.covers(now);
        Self {
            sound: preferences.alert_sound && may_interrupt,
            windows_notification: preferences.windows_notifications && may_interrupt,
        }
    }
}
