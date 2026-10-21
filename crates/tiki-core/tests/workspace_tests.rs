mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::capture::{CaptureIntent, CaptureOutcome, ConfirmedCapture};
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
fn capturing_a_sentence_creates_a_task_for_the_client() {
    let workspace = sample_workspace();
    workspace
        .capture("revisar permisos de Vapestation mañana")
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let due_soon = snapshot
        .sections
        .iter()
        .find(|section| section.key == "due_soon")
        .unwrap();
    assert!(due_soon.items.iter().any(|item| {
        item.title == "Revisar permisos" && item.client_name.as_deref() == Some("Vapestation")
    }));
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
    workspace.set_focus_mode(true);
    assert_eq!(workspace.island_snapshot().unwrap().mood_label, "ZZZ");
}

#[test]
fn a_plus_sign_creates_a_client_that_capture_then_recognizes() {
    let workspace = sample_workspace();

    let outcome = workspace.capture("+Constructora").unwrap();
    assert!(matches!(outcome, CaptureOutcome::Client(_)));

    let intent = workspace
        .preview_capture("visitar constructora jueves")
        .unwrap();
    let CaptureIntent::Task(draft) = intent else {
        panic!("expected a task");
    };
    assert_eq!(draft.client_name.as_deref(), Some("Constructora"));
    assert_eq!(draft.due_date, Some(TestDay::october(22)));
}

#[test]
fn a_plus_sign_alone_is_rejected() {
    assert!(sample_workspace().capture("+   ").is_err());
}

#[test]
fn preferences_are_saved_and_applied() {
    let mut workspace = sample_workspace();
    workspace
        .update_preferences(UserPreferences {
            locale: Locale::English,
            island_placement: IslandPlacement::Right,
        })
        .unwrap();

    assert_eq!(
        workspace.preferences().island_placement,
        IslandPlacement::Right
    );
    assert_eq!(workspace.island_snapshot().unwrap().mood_label, "LATE");
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
    });
    assert!(saved.is_ok());
}
