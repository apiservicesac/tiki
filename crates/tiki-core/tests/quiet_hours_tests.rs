mod support;

use chrono::NaiveTime;
use support::{SampleWorkspace, TestDay, TimedWorkspace};
use tiki_core::TikiSettings;
use tiki_core::mood::Mood;
use tiki_core::preferences::{QuietHours, UserPreferences};

fn at(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

fn quiet_hours(from: NaiveTime, until: NaiveTime) -> QuietHours {
    QuietHours {
        enabled: true,
        from,
        until,
    }
}

#[test]
fn a_night_across_midnight_covers_both_sides_of_it() {
    let night = quiet_hours(at(22, 0), at(7, 0));

    assert!(night.covers(at(22, 0)));
    assert!(night.covers(at(23, 59)));
    assert!(night.covers(at(0, 0)));
    assert!(night.covers(at(6, 59)));
    assert!(!night.covers(at(7, 0)));
    assert!(!night.covers(at(12, 0)));
    assert!(!night.covers(at(21, 59)));
}

#[test]
fn a_range_within_one_day_covers_only_its_hours() {
    let siesta = quiet_hours(at(13, 0), at(15, 0));

    assert!(siesta.covers(at(13, 0)));
    assert!(siesta.covers(at(14, 30)));
    assert!(!siesta.covers(at(15, 0)));
    assert!(!siesta.covers(at(0, 0)));
}

#[test]
fn quiet_hours_switched_off_or_empty_never_apply() {
    let off = QuietHours {
        enabled: false,
        ..quiet_hours(at(22, 0), at(7, 0))
    };
    let empty = quiet_hours(at(22, 0), at(22, 0));

    assert!(!off.covers(at(23, 0)));
    assert!(!empty.covers(at(22, 0)));
    assert!(!empty.covers(at(3, 0)));
}

#[test]
fn a_new_user_gets_the_night_from_the_settings() {
    let settings = TikiSettings::default();
    let preferences = UserPreferences::defaults(&settings);

    assert_eq!(preferences.quiet_hours, settings.quiet_hours);
    assert!(preferences.quiet_hours.covers(at(23, 0)));
    assert!(!preferences.quiet_hours.covers(at(10, 0)));
}

#[test]
fn quiet_hours_survive_a_restart() {
    let sample = SampleWorkspace::create();
    let defaults = UserPreferences::defaults(&TikiSettings::default());
    let custom = UserPreferences {
        quiet_hours: QuietHours {
            enabled: false,
            ..quiet_hours(at(23, 30), at(6, 15))
        },
        ..defaults.clone()
    };
    sample.database.preferences().save(&custom).unwrap();

    let loaded = sample.database.preferences().load(defaults).unwrap();
    assert_eq!(loaded.quiet_hours, custom.quiet_hours);
}

#[test]
fn quiet_hours_accept_times_as_the_time_picker_writes_them() {
    let mut json =
        serde_json::to_value(UserPreferences::defaults(&TikiSettings::default())).unwrap();
    json["quietHours"]["from"] = "23:15".into();

    let preferences: UserPreferences = serde_json::from_value(json).unwrap();
    assert_eq!(preferences.quiet_hours.from, at(23, 15));
}

#[test]
fn tiki_sleeps_through_the_night_and_wakes_up_to_the_overdue_work() {
    let workspace = TimedWorkspace::sample_at(TestDay::today().and_time(at(23, 0)));
    assert_eq!(workspace.island_snapshot().unwrap().mood, Mood::Sleep);

    let morning = workspace.at(TestDay::working_hours());
    assert_eq!(morning.island_snapshot().unwrap().mood, Mood::Worried);
}
