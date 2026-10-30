mod support;

use chrono::{Duration, NaiveDateTime, NaiveTime};
use support::TestDay;
use tiki_core::calendar::MeetingItem;
use tiki_core::sessions::{PresenceSignals, RecordingCue, RecordingSuggestionPolicy};

const FRESH_MEETING_MINUTES: i64 = 5;

fn at(hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::today().and_time(NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time"))
}

fn meeting(title: &str, starts_at: NaiveDateTime) -> MeetingItem {
    MeetingItem {
        title: title.to_owned(),
        starts_at,
        ends_at: starts_at + Duration::hours(1),
        all_day: false,
        location: String::new(),
        join_url: None,
        account_email: "ana@example.com".to_owned(),
    }
}

/// What the policy offers for one reading of the signals.
struct Observer {
    policy: RecordingSuggestionPolicy,
}

impl Observer {
    fn new() -> Self {
        Self {
            policy: RecordingSuggestionPolicy::default(),
        }
    }

    fn observe(
        &mut self,
        now: NaiveDateTime,
        in_call: bool,
        recording: bool,
        meetings: &[MeetingItem],
    ) -> Option<RecordingCue> {
        self.policy.observe(
            &PresenceSignals {
                now,
                in_call,
                recording,
                meetings_in_progress: meetings,
            },
            Duration::minutes(FRESH_MEETING_MINUTES),
        )
    }
}

#[test]
fn a_call_is_offered_once_while_it_lasts() {
    let mut observer = Observer::new();
    assert_eq!(
        observer.observe(at(10, 0), true, false, &[]),
        Some(RecordingCue::Call)
    );
    assert_eq!(observer.observe(at(10, 1), true, false, &[]), None);
    assert_eq!(observer.observe(at(10, 30), false, false, &[]), None);
    assert_eq!(
        observer.observe(at(11, 0), true, false, &[]),
        Some(RecordingCue::Call)
    );
}

#[test]
fn nothing_is_offered_while_recording_nor_after_it_in_the_same_call() {
    let mut observer = Observer::new();
    assert_eq!(observer.observe(at(10, 0), true, true, &[]), None);
    assert_eq!(observer.observe(at(10, 20), true, false, &[]), None);
}

#[test]
fn a_meeting_is_offered_once_when_it_begins() {
    let mut observer = Observer::new();
    let review = [meeting("Revisión", at(10, 0))];
    assert_eq!(
        observer.observe(at(10, 1), false, false, &review),
        Some(RecordingCue::Meeting {
            title: "Revisión".to_owned()
        })
    );
    assert_eq!(observer.observe(at(10, 2), false, false, &review), None);
}

#[test]
fn joining_the_call_of_an_offered_meeting_is_the_same_occasion() {
    let mut observer = Observer::new();
    let review = [meeting("Revisión", at(10, 0))];
    assert!(observer.observe(at(10, 0), false, false, &review).is_some());
    assert_eq!(observer.observe(at(10, 2), true, false, &review), None);
}

#[test]
fn a_meeting_beginning_during_an_offered_call_is_not_offered_again() {
    let mut observer = Observer::new();
    assert_eq!(
        observer.observe(at(9, 58), true, false, &[]),
        Some(RecordingCue::Call)
    );
    let review = [meeting("Revisión", at(10, 0))];
    assert_eq!(observer.observe(at(10, 0), true, false, &review), None);
}

#[test]
fn a_meeting_well_under_way_is_not_offered() {
    let mut observer = Observer::new();
    let review = [meeting("Revisión", at(10, 0))];
    assert_eq!(observer.observe(at(10, 30), false, false, &review), None);
}
