mod support;

use chrono::NaiveTime;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::ai::AiProvider;
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::{Priority, TaskStatus};
use tiki_core::i18n::Locale;
use tiki_core::mood::Mood;
use tiki_core::preferences::{IslandPlacement, UserPreferences};
use tiki_core::workspace::{FixedClock, TikiWorkspace, Urgency};

fn sample_workspace() -> TikiWorkspace {
    let sample = SampleWorkspace::create();
    TikiWorkspace::new(
        sample.database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap()
}

#[test]
fn the_snapshot_is_translated_and_ready_to_draw() {
    let snapshot = sample_workspace().island_snapshot().unwrap();

    assert_eq!(snapshot.mood, Mood::Worried);
    assert_eq!(snapshot.mood_label, "ATRASO");
    assert_eq!(snapshot.attention_count, 5);
    assert_eq!(snapshot.due_count, 3);
    assert_eq!(snapshot.waiting_count, 2);
    assert_eq!(snapshot.closed_summary, "3 hoy · 2 esperando");
    assert_eq!(
        snapshot.today_subtitle,
        "miércoles 21 de octubre · 3 vencen pronto"
    );

    let keys: Vec<&str> = snapshot
        .sections
        .iter()
        .map(|section| section.key.as_str())
        .collect();
    assert_eq!(keys, ["overdue", "due_soon", "your_turn", "waiting"]);

    let due_soon = &snapshot.sections[1];
    assert_eq!(due_soon.title, "Vence pronto");
    assert_eq!(due_soon.items[0].detail_label, "mañana");
    assert_eq!(due_soon.items[0].client_color.as_deref(), Some("#2F6BD8"));
    assert_eq!(due_soon.items[0].urgency, Urgency::Soon);
}

#[test]
fn switching_to_english_translates_the_snapshot() {
    let mut workspace = sample_workspace();
    workspace.change_locale(Locale::English).unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    assert_eq!(snapshot.mood_label, "LATE");
    assert_eq!(snapshot.sections[1].title, "Due soon");
}

#[test]
fn delivering_the_overdue_task_calms_tiki_down_to_hurry() {
    let workspace = sample_workspace();
    let overdue_id = workspace.island_snapshot().unwrap().sections[0].items[0].task_id;

    workspace.move_task(overdue_id, TaskStatus::Done).unwrap();

    assert_eq!(workspace.island_snapshot().unwrap().mood, Mood::Hurry);
}

#[test]
fn focus_mode_puts_tiki_to_sleep() {
    let mut workspace = sample_workspace();
    workspace.set_focus_mode(true).unwrap();
    assert_eq!(workspace.island_snapshot().unwrap().mood_label, "ZZZ");
}

#[test]
fn preferences_are_saved_and_applied() {
    let mut workspace = sample_workspace();
    workspace
        .update_preferences(UserPreferences {
            locale: Locale::English,
            island_placement: IslandPlacement::Right,
            ai_provider: AiProvider::OpenAi,
            ai_model: Some("gpt-5.5".into()),
            morning_brief_time: NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
            notifications_enabled: false,
            ..UserPreferences::defaults(&TikiSettings::default())
        })
        .unwrap();

    assert_eq!(workspace.preferences().ai_provider, AiProvider::OpenAi);
    assert_eq!(workspace.preferences().ai_model.as_deref(), Some("gpt-5.5"));
    assert_eq!(
        workspace.preferences().island_placement,
        IslandPlacement::Right
    );
    assert_eq!(workspace.island_snapshot().unwrap().mood_label, "LATE");
    assert!(!workspace.preferences().allows_notifications());
}

#[test]
fn a_confirmed_capture_is_saved_with_its_client_priority_and_date() {
    let workspace = sample_workspace();
    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Revisar acceso de Almendra".into(),
            client_name: Some("andeanvet".into()),
            due_date: Some(TestDay::october(22)),
            priority: Priority::High,
            remind_at: None,
        })
        .unwrap();

    let due_soon = workspace.island_snapshot().unwrap().sections[1].clone();
    let saved = due_soon
        .items
        .iter()
        .find(|item| item.title == "Revisar acceso de Almendra")
        .unwrap();
    assert_eq!(saved.client_name.as_deref(), Some("Andeanvet"));
}

#[test]
fn an_unknown_client_name_is_left_out_instead_of_failing() {
    let workspace = sample_workspace();
    let saved = workspace.save_confirmed(ConfirmedCapture {
        title: "Llamar".into(),
        client_name: Some("Cliente que no existe".into()),
        due_date: None,
        priority: Priority::Normal,
        remind_at: None,
    });
    assert!(saved.is_ok());
}

#[test]
fn the_assistant_sees_open_tasks_without_ids_or_notes() {
    let context = sample_workspace().assistant_context().unwrap();

    assert_eq!(context.today, TestDay::today());
    assert_eq!(context.open_tasks.len(), 7);
    let waiting = context
        .open_tasks
        .iter()
        .find(|task| task.status == TaskStatus::WaitingOnOthers)
        .unwrap();
    assert!(waiting.waiting_days.is_some());

    let json = serde_json::to_string(&context).unwrap();
    assert!(!json.contains("\"id\""));
    assert!(!json.contains("notes"));
}
