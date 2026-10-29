use chrono::{Duration, NaiveDateTime};

use crate::calendar::MeetingItem;

/// What makes Tiki offer to record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingCue {
    /// Another app took the microphone: a call started.
    Call,
    /// A calendar meeting began.
    Meeting { title: String },
}

/// What the presence signals say right now, read by the policy.
#[derive(Debug, Clone, Copy)]
pub struct PresenceSignals<'meetings> {
    pub now: NaiveDateTime,
    pub in_call: bool,
    /// A session is recording or paused.
    pub recording: bool,
    /// Timed meetings in progress now, earliest first.
    pub meetings_in_progress: &'meetings [MeetingItem],
}

/// Decides when Tiki offers to record: once when a call starts and once when a meeting
/// begins, never while recording, and never twice for the same occasion (a meeting that
/// begins during a call already offered is the same occasion). Tiki only offers: the user
/// starts the recording with a click.
#[derive(Debug, Default)]
pub struct RecordingSuggestionPolicy {
    /// The call in progress was offered (or recorded).
    call_handled: bool,
    /// Meetings in progress already offered (or recorded), by title and start.
    handled_meetings: Vec<(String, NaiveDateTime)>,
}

impl RecordingSuggestionPolicy {
    /// The offer to make now, if any; `fresh_meeting` is how long after its start a meeting
    /// still counts as beginning.
    pub fn observe(
        &mut self,
        signals: &PresenceSignals<'_>,
        fresh_meeting: Duration,
    ) -> Option<RecordingCue> {
        self.handled_meetings.retain(|(title, starts_at)| {
            signals
                .meetings_in_progress
                .iter()
                .any(|meeting| &meeting.title == title && meeting.starts_at == *starts_at)
        });
        if !signals.in_call {
            self.call_handled = false;
        }
        let new_meeting = signals
            .meetings_in_progress
            .iter()
            .find(|meeting| {
                signals.now - meeting.starts_at <= fresh_meeting && !self.was_handled(meeting)
            })
            .cloned();
        // Joining the call of a meeting already offered is the same occasion.
        let meeting_already_offered = signals
            .meetings_in_progress
            .iter()
            .any(|meeting| self.was_handled(meeting));
        let cue = if signals.recording {
            None
        } else if signals.in_call {
            // During a call its occasion is offered once, whatever begins meanwhile.
            (!self.call_handled && !meeting_already_offered).then_some(RecordingCue::Call)
        } else {
            new_meeting.map(|meeting| RecordingCue::Meeting {
                title: meeting.title,
            })
        };
        self.call_handled |= signals.in_call;
        for meeting in signals.meetings_in_progress {
            if !self.was_handled(meeting) {
                self.handled_meetings
                    .push((meeting.title.clone(), meeting.starts_at));
            }
        }
        cue
    }

    fn was_handled(&self, meeting: &MeetingItem) -> bool {
        self.handled_meetings
            .iter()
            .any(|(title, starts_at)| &meeting.title == title && meeting.starts_at == *starts_at)
    }
}
