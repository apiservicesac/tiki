mod support;

use chrono::{NaiveDateTime, NaiveTime};
use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::briefing::BriefMoment;
use tiki_core::preferences::UserPreferences;
use tiki_core::reminders::ReminderPlanner;
use tiki_core::storage::Database;
use tiki_core::workspace::{FixedClock, TikiWorkspace};

fn sample_workspace() -> TikiWorkspace {
    TikiWorkspace::new(
        SampleWorkspace::create().database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap()
}

fn today_at(hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::today().and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
}

#[test]
fn the_morning_brief_waits_for_its_time() {
    let preferences = UserPreferences::defaults(&TikiSettings::default());
    let planner = ReminderPlanner::new(&preferences);

    assert_eq!(planner.due_moment(today_at(8, 0), |_| None), None);
    assert_eq!(
        planner.due_moment(today_at(8, 30), |_| None),
        Some(BriefMoment::Morning)
    );
    assert_eq!(
        planner.due_moment(today_at(11, 0), |_| Some(TestDay::october(20))),
        Some(BriefMoment::Morning)
    );
    assert_eq!(
        planner.due_moment(today_at(11, 0), |_| Some(TestDay::today())),
        None
    );
    assert_eq!(
        planner.due_moment(today_at(18, 5), |_| None),
        Some(BriefMoment::Evening)
    );
}

#[test]
fn each_brief_arrives_once_a_day_with_counts_and_follow_ups() {
    let workspace = sample_workspace();

    let morning = workspace
        .take_due_notification(today_at(9, 0))
        .unwrap()
        .unwrap();
    assert_eq!(morning.moment, BriefMoment::Morning);
    assert_eq!(morning.title, "Buenos días · tu día con Tiki");
    assert!(
        morning
            .body
            .starts_with("Atrasado: 1 · Vence pronto: 2 · Esperando: 2")
    );
    assert!(morning.body.contains("Empieza por: "));
    assert!(morning.body.contains("Haz seguimiento: "));

    assert!(
        workspace
            .take_due_notification(today_at(10, 0))
            .unwrap()
            .is_none()
    );

    let evening = workspace
        .take_due_notification(today_at(18, 0))
        .unwrap()
        .unwrap();
    assert_eq!(evening.title, "Cierre del día");
    assert!(evening.body.contains("Mañana vence: "));
}

#[test]
fn focus_mode_and_disabled_notifications_stay_quiet() {
    let mut workspace = sample_workspace();
    workspace.set_focus_mode(true).unwrap();
    assert!(
        workspace
            .take_due_notification(today_at(9, 0))
            .unwrap()
            .is_none()
    );

    workspace.set_focus_mode(false).unwrap();
    workspace
        .update_preferences(UserPreferences {
            notifications_enabled: false,
            ..workspace.preferences().clone()
        })
        .unwrap();
    assert!(
        workspace
            .take_due_notification(today_at(9, 0))
            .unwrap()
            .is_none()
    );
}

#[test]
fn brief_times_survive_a_restart() {
    let sample = SampleWorkspace::create();
    let custom = UserPreferences {
        morning_brief_time: NaiveTime::from_hms_opt(7, 45, 0).unwrap(),
        evening_brief_time: NaiveTime::from_hms_opt(17, 30, 0).unwrap(),
        focus_mode: true,
        ..UserPreferences::defaults(&TikiSettings::default())
    };
    sample.database.preferences().save(&custom).unwrap();

    let loaded = sample
        .database
        .preferences()
        .load(UserPreferences::defaults(&TikiSettings::default()))
        .unwrap();
    assert_eq!(loaded, custom);
}

#[test]
fn preferences_accept_times_as_the_time_picker_writes_them() {
    let mut json =
        serde_json::to_value(UserPreferences::defaults(&TikiSettings::default())).unwrap();
    json["morningBriefTime"] = "07:15".into();

    let preferences: UserPreferences = serde_json::from_value(json).unwrap();
    assert_eq!(
        preferences.morning_brief_time,
        NaiveTime::from_hms_opt(7, 15, 0).unwrap()
    );
}

#[test]
fn moving_a_brief_later_today_sends_it_again_at_the_new_time() {
    let sample = SampleWorkspace::create();
    let database: Database = sample.database;
    let mut workspace = TikiWorkspace::new(
        database,
        TikiSettings::default(),
        Box::new(FixedClock::at(today_at(17, 29))),
    )
    .unwrap();
    assert!(
        workspace
            .take_due_notification(today_at(17, 0))
            .unwrap()
            .is_some()
    );

    workspace
        .update_preferences(UserPreferences {
            morning_brief_time: NaiveTime::from_hms_opt(17, 30, 0).unwrap(),
            ..workspace.preferences().clone()
        })
        .unwrap();

    assert!(
        workspace
            .take_due_notification(today_at(17, 29))
            .unwrap()
            .is_none()
    );
    let again = workspace
        .take_due_notification(today_at(17, 30))
        .unwrap()
        .unwrap();
    assert_eq!(again.moment, BriefMoment::Morning);
}

#[test]
fn moving_a_brief_to_a_time_already_past_waits_for_tomorrow() {
    let mut workspace = TikiWorkspace::new(
        SampleWorkspace::create().database,
        TikiSettings::default(),
        Box::new(FixedClock::at(today_at(17, 29))),
    )
    .unwrap();
    workspace.take_due_notification(today_at(17, 0)).unwrap();

    workspace
        .update_preferences(UserPreferences {
            morning_brief_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            ..workspace.preferences().clone()
        })
        .unwrap();
    assert!(
        workspace
            .take_due_notification(today_at(17, 31))
            .unwrap()
            .is_none()
    );
}

#[test]
fn the_test_notification_does_not_use_up_the_brief() {
    let workspace = sample_workspace();
    let preview = workspace
        .preview_notification(BriefMoment::Morning)
        .unwrap();
    assert!(preview.body.contains("Vence pronto"));
    assert!(
        workspace
            .take_due_notification(today_at(9, 0))
            .unwrap()
            .is_some()
    );
}
