mod support;

use tiki_core::domain::ItemKind;

use chrono::{NaiveDateTime, NaiveTime};
use support::{SampleWorkspace, TestDay, TimedWorkspace};
use tiki_core::TikiSettings;
use tiki_core::alerts::{AlertKind, BriefPlanner};
use tiki_core::briefing::BriefMoment;
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::Priority;
use tiki_core::preferences::UserPreferences;

fn sample_workspace() -> TimedWorkspace {
    TimedWorkspace::sample_at(today_at(0, 0))
}

fn today_at(hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::today().and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
}

#[test]
fn the_morning_brief_waits_for_its_time() {
    let preferences = UserPreferences::defaults(&TikiSettings::default());
    let planner = BriefPlanner::new(&preferences);

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
        .at(today_at(9, 0))
        .take_due_notification()
        .unwrap()
        .unwrap();
    assert_eq!(morning.kind, AlertKind::Morning);
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
            .at(today_at(10, 0))
            .take_due_notification()
            .unwrap()
            .is_none()
    );

    let evening = workspace
        .at(today_at(18, 0))
        .take_due_notification()
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
            .at(today_at(9, 0))
            .take_due_notification()
            .unwrap()
            .is_none()
    );

    workspace.set_focus_mode(false).unwrap();
    let changed = UserPreferences {
        notifications_enabled: false,
        ..workspace.preferences().clone()
    };
    workspace.update_preferences(changed).unwrap();
    assert!(
        workspace
            .at(today_at(9, 0))
            .take_due_notification()
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
        alert_sound: false,
        windows_notifications: false,
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
    let mut workspace = TimedWorkspace::sample_at(today_at(17, 29));
    assert!(
        workspace
            .at(today_at(17, 0))
            .take_due_notification()
            .unwrap()
            .is_some()
    );

    let changed = UserPreferences {
        morning_brief_time: NaiveTime::from_hms_opt(17, 30, 0).unwrap(),
        ..workspace.preferences().clone()
    };

    workspace.update_preferences(changed).unwrap();

    assert!(
        workspace
            .at(today_at(17, 29))
            .take_due_notification()
            .unwrap()
            .is_none()
    );
    let again = workspace
        .at(today_at(17, 30))
        .take_due_notification()
        .unwrap()
        .unwrap();
    assert_eq!(again.kind, AlertKind::Morning);
}

#[test]
fn moving_a_brief_to_a_time_already_past_waits_for_tomorrow() {
    let mut workspace = TimedWorkspace::sample_at(today_at(17, 29));
    workspace
        .at(today_at(17, 0))
        .take_due_notification()
        .unwrap();

    let changed = UserPreferences {
        morning_brief_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        ..workspace.preferences().clone()
    };

    workspace.update_preferences(changed).unwrap();
    assert!(
        workspace
            .at(today_at(17, 31))
            .take_due_notification()
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_task_reminder_rings_once_at_its_time() {
    let workspace = sample_workspace();
    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Llamar".into(),
            project_name: Some("Altamira".into()),
            due_date: Some(TestDay::october(22)),
            priority: Priority::Normal,
            remind_at: Some(TestDay::october(22).and_hms_opt(10, 0, 0).unwrap()),
            kind: ItemKind::Task,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: None,
        })
        .unwrap();

    let before_ten = TestDay::october(22).and_time(NaiveTime::from_hms_opt(9, 59, 0).unwrap());
    assert_eq!(
        workspace.at(before_ten).take_due_task_reminders().unwrap(),
        Vec::new()
    );
    let ten = TestDay::october(22).and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
    let alerts = workspace.at(ten).take_due_task_reminders().unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::Task);
    assert_eq!(alerts[0].title, "Aviso de tarea");
    assert!(alerts[0].body.starts_with("Llamar\nAltamira · "));
    assert!(alerts[0].task_id.is_some());

    assert_eq!(
        workspace.at(ten).take_due_task_reminders().unwrap(),
        Vec::new()
    );
}

fn reminder(title: &str, at: NaiveDateTime, repeat: tiki_core::domain::Repeat) -> ConfirmedCapture {
    ConfirmedCapture {
        title: title.into(),
        project_name: None,
        due_date: None,
        priority: Priority::Normal,
        remind_at: Some(at),
        kind: tiki_core::domain::ItemKind::Reminder,
        tags: Vec::new(),
        repeat,
        waiting_for: None,
    }
}

#[test]
fn a_repeating_reminder_comes_back_and_a_one_time_reminder_is_done() {
    use tiki_core::domain::Repeat;
    let workspace = sample_workspace();
    let pill = workspace
        .save_confirmed(reminder("Tomar la pastilla", today_at(8, 0), Repeat::Daily))
        .unwrap();
    workspace
        .save_confirmed(reminder("Llamar a mamá", today_at(17, 0), Repeat::Never))
        .unwrap();

    let alerts = workspace
        .at(today_at(18, 0))
        .take_due_task_reminders()
        .unwrap();
    assert_eq!(alerts.len(), 2);
    assert!(alerts.iter().all(|alert| alert.kind == AlertKind::Reminder));
    assert_eq!(alerts[0].title, "Recordatorio");

    let pill_details = workspace.task_details(pill).unwrap();
    assert_eq!(
        pill_details.remind_at,
        Some(TestDay::october(22).and_hms_opt(8, 0, 0).unwrap())
    );
    assert_eq!(workspace.island_snapshot().unwrap().delivered_today, 1);
}

#[test]
fn snoozing_a_reminder_that_rang_brings_it_back() {
    use tiki_core::domain::{Repeat, TaskStatus};
    let workspace = TimedWorkspace::sample_at(today_at(17, 1));
    let call = workspace
        .save_confirmed(reminder("Llamar a mamá", today_at(17, 0), Repeat::Never))
        .unwrap();
    workspace
        .at(today_at(17, 1))
        .take_due_task_reminders()
        .unwrap();
    assert_eq!(
        workspace.task_details(call).unwrap().status,
        TaskStatus::Done
    );

    workspace.snooze(call, 10).unwrap();
    let details = workspace.task_details(call).unwrap();
    assert_eq!(details.status, TaskStatus::ToDo);
    assert_eq!(details.remind_at, Some(today_at(17, 11)));
}

#[test]
fn waiting_on_someone_schedules_a_follow_up_reminder() {
    let workspace = sample_workspace();
    let contract = workspace
        .save_confirmed(ConfirmedCapture {
            title: "Contrato firmado".into(),
            project_name: Some("Constructora Sur".into()),
            due_date: None,
            priority: Priority::Normal,
            remind_at: None,
            kind: tiki_core::domain::ItemKind::Task,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: Some(" respuesta de Lucía ".into()),
        })
        .unwrap();

    let details = workspace.task_details(contract).unwrap();
    assert_eq!(details.status, tiki_core::domain::TaskStatus::Waiting);
    assert_eq!(details.waiting_for, "respuesta de Lucía");
    assert_eq!(details.project_name.as_deref(), Some("Constructora Sur"));
    assert_eq!(
        details.remind_at,
        Some(TestDay::october(24).and_hms_opt(9, 0, 0).unwrap())
    );

    let alerts = workspace
        .at(TestDay::october(24).and_hms_opt(9, 0, 0).unwrap())
        .take_due_task_reminders()
        .unwrap();
    let follow_up = alerts
        .iter()
        .find(|alert| alert.task_id == Some(contract))
        .unwrap();
    assert_eq!(follow_up.kind, AlertKind::FollowUp);
    assert_eq!(follow_up.title, "Toca insistir");
    assert!(
        follow_up
            .body
            .contains("Esperando: respuesta de Lucía · hace 3 días")
    );
}

#[test]
fn weekday_repeats_skip_the_weekend() {
    use tiki_core::domain::Repeat;
    let friday = TestDay::october(23).and_hms_opt(9, 0, 0).unwrap();
    assert_eq!(
        Repeat::Weekdays.next_after(friday, friday),
        Some(TestDay::october(26).and_hms_opt(9, 0, 0).unwrap())
    );
    assert_eq!(Repeat::Never.next_after(friday, friday), None);
}

#[test]
fn a_reminder_shows_its_moment_and_how_often_it_repeats() {
    use tiki_core::domain::Repeat;
    let workspace = sample_workspace();
    workspace
        .save_confirmed(reminder(
            "Tomar agua",
            TestDay::october(23).and_hms_opt(11, 0, 0).unwrap(),
            Repeat::Daily,
        ))
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let water = snapshot
        .open_items
        .iter()
        .find(|item| item.title == "Tomar agua")
        .unwrap();
    assert_eq!(water.detail_label, "Vie 23 · 11:00 · Cada día");
    // Not today: it is in the list of reminders, not in Today.
    assert!(
        snapshot
            .sections
            .iter()
            .all(|section| section.key != "reminders")
    );
}
