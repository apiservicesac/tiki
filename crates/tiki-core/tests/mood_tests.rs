mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::briefing::{DailyBrief, DailyBriefBuilder};
use tiki_core::domain::ProjectDirectory;
use tiki_core::mood::{Mood, MoodResolver, MoodSignals};

fn resolve(brief: &DailyBrief, minutes_to_next_meeting: Option<i64>, focus_mode: bool) -> Mood {
    let settings = TikiSettings::default();
    MoodResolver::new(&settings).resolve(&MoodSignals {
        brief,
        minutes_to_next_meeting,
        focus_mode,
        quiet_hours: false,
    })
}

fn sample_brief() -> DailyBrief {
    let settings = TikiSettings::default();
    let workspace = SampleWorkspace::create();
    let tasks = workspace.database.tasks().list_open().unwrap();
    DailyBriefBuilder::new(&settings).build(&tasks, &workspace.directory(), TestDay::today())
}

fn empty_brief() -> DailyBrief {
    let settings = TikiSettings::default();
    DailyBriefBuilder::new(&settings).build(&[], &ProjectDirectory::default(), TestDay::today())
}

#[test]
fn an_empty_day_is_calm() {
    assert_eq!(resolve(&empty_brief(), None, false), Mood::Calm);
}

#[test]
fn an_overdue_task_worries_tiki() {
    assert_eq!(resolve(&sample_brief(), None, false), Mood::Worried);
}

#[test]
fn a_close_meeting_beats_overdue_work() {
    assert_eq!(resolve(&sample_brief(), Some(10), false), Mood::Alert);
    assert_eq!(resolve(&sample_brief(), Some(45), false), Mood::Worried);
}

#[test]
fn focus_mode_always_sleeps() {
    assert_eq!(resolve(&sample_brief(), Some(5), true), Mood::Sleep);
}

#[test]
fn the_night_puts_tiki_to_sleep_even_with_overdue_work_and_a_meeting() {
    let settings = TikiSettings::default();
    let mood = MoodResolver::new(&settings).resolve(&MoodSignals {
        brief: &sample_brief(),
        minutes_to_next_meeting: Some(5),
        focus_mode: false,
        quiet_hours: true,
    });
    assert_eq!(mood, Mood::Sleep);
}
