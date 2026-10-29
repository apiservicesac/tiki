mod support;

use chrono::NaiveTime;
use support::{TestDay, TimedWorkspace};
use tiki_core::TikiSettings;
use tiki_core::alerts::{AlertDelivery, AlertKind};
use tiki_core::preferences::{QuietHours, UserPreferences};

const SILENT: AlertDelivery = AlertDelivery {
    sound: false,
    windows_notification: false,
};

fn at(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

fn defaults() -> UserPreferences {
    UserPreferences::defaults(&TikiSettings::default())
}

#[test]
fn during_the_day_an_alert_sounds_and_reaches_windows_as_chosen() {
    assert_eq!(
        AlertDelivery::decide(&defaults(), at(10, 0), false),
        AlertDelivery {
            sound: true,
            windows_notification: true,
        }
    );

    let without_sound = UserPreferences {
        alert_sound: false,
        ..defaults()
    };
    assert_eq!(
        AlertDelivery::decide(&without_sound, at(10, 0), false),
        AlertDelivery {
            sound: false,
            windows_notification: true,
        }
    );

    let island_only = UserPreferences {
        windows_notifications: false,
        ..defaults()
    };
    assert_eq!(
        AlertDelivery::decide(&island_only, at(10, 0), false),
        AlertDelivery {
            sound: true,
            windows_notification: false,
        }
    );
}

#[test]
fn at_night_and_during_a_call_alerts_make_no_sound() {
    assert_eq!(AlertDelivery::decide(&defaults(), at(23, 0), false), SILENT);
    assert_eq!(AlertDelivery::decide(&defaults(), at(6, 30), false), SILENT);
    assert_eq!(AlertDelivery::decide(&defaults(), at(10, 0), true), SILENT);
}

#[test]
fn with_quiet_hours_off_the_night_sounds_like_the_day() {
    let awake = UserPreferences {
        quiet_hours: QuietHours {
            enabled: false,
            ..defaults().quiet_hours
        },
        ..defaults()
    };
    assert_eq!(
        AlertDelivery::decide(&awake, at(23, 0), false),
        AlertDelivery::decide(&awake, at(10, 0), false)
    );
}

#[test]
fn a_brief_during_a_call_still_arrives_but_silently() {
    let mut workspace = TimedWorkspace::sample_at(TestDay::today().and_time(at(9, 0)));
    workspace.set_in_call(true);

    let due = workspace.take_due_alerts().unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].alert.kind, AlertKind::Morning);
    assert_eq!(due[0].delivery, SILENT);
}

#[test]
fn the_island_knows_a_call_is_on_until_it_ends() {
    let mut workspace = TimedWorkspace::sample_at(TestDay::working_hours());
    assert!(!workspace.island_snapshot().unwrap().in_call);

    workspace.set_in_call(true);
    assert!(workspace.island_snapshot().unwrap().in_call);

    workspace.set_in_call(false);
    assert!(!workspace.island_snapshot().unwrap().in_call);
}

#[test]
fn a_brief_after_the_call_sounds_again() {
    let mut workspace = TimedWorkspace::sample_at(TestDay::today().and_time(at(9, 0)));
    workspace.set_in_call(true);
    workspace.set_in_call(false);

    let due = workspace.take_due_alerts().unwrap();
    assert!(due[0].delivery.sound);
    assert!(due[0].delivery.windows_notification);
}

#[test]
fn a_brief_inside_the_quiet_hours_arrives_silently() {
    let mut workspace = TimedWorkspace::sample_at(TestDay::today().and_time(at(9, 0)));
    let late_sleeper = UserPreferences {
        quiet_hours: QuietHours {
            enabled: true,
            from: at(0, 0),
            until: at(10, 0),
        },
        ..workspace.preferences().clone()
    };
    workspace.update_preferences(late_sleeper).unwrap();

    let due = workspace.take_due_alerts().unwrap();
    assert_eq!(due[0].alert.kind, AlertKind::Morning);
    assert_eq!(due[0].delivery, SILENT);
}

#[test]
fn the_island_receives_the_alert_with_its_delivery() {
    let workspace = TimedWorkspace::sample_at(TestDay::today().and_time(at(9, 0)));

    let json = serde_json::to_value(&workspace.take_due_alerts().unwrap()[0]).unwrap();
    assert_eq!(json["kind"], "morning");
    assert_eq!(json["delivery"]["sound"], true);
    assert_eq!(json["delivery"]["windowsNotification"], true);
}
