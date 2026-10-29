mod support;

use tiki_core::domain::ItemKind;

use chrono::NaiveTime;

use support::{SampleWorkspace, TestDay, TimedWorkspace};
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
        Box::new(FixedClock::at(TestDay::working_hours())),
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
    // Today only shows what presses today; every open item is in `open_items`.
    assert_eq!(keys, ["overdue", "due_soon", "follow_up"]);
    assert_eq!(snapshot.open_items.len(), 7);
    assert_eq!(snapshot.open_count, 7);
    let focus = snapshot.focus.as_ref().unwrap();
    assert_eq!(focus.reason, "Está atrasada");
    assert_eq!(focus.item.title, snapshot.sections[0].items[0].title);

    let due_soon = &snapshot.sections[1];
    assert_eq!(due_soon.title, "Vence pronto");
    assert_eq!(due_soon.items[0].detail_label, "mañana");
    assert_eq!(due_soon.items[0].project_color.as_deref(), Some("#2F6BD8"));
    assert_eq!(due_soon.items[0].urgency, Urgency::Soon);
}

#[test]
fn snapshot_items_say_what_they_are_and_which_project_they_belong_to() {
    let sample = SampleWorkspace::create();
    let altamira = sample.altamira;
    let workspace = TimedWorkspace::open(sample.database, TestDay::start());
    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Tomar agua".into(),
            project_name: None,
            due_date: None,
            priority: Priority::Normal,
            remind_at: Some(TestDay::october(23).and_hms_opt(11, 0, 0).unwrap()),
            kind: ItemKind::Reminder,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: None,
        })
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let item = |title: &str| {
        snapshot
            .open_items
            .iter()
            .find(|item| item.title == title)
            .unwrap()
    };
    assert_eq!(item("Tomar agua").kind, ItemKind::Reminder);
    assert_eq!(item("Tomar agua").project_id, None);
    let tracking = item("Tracking de formularios comerciales");
    assert_eq!(tracking.kind, ItemKind::Task);
    assert_eq!(tracking.project_id, Some(altamira));
    // The reminder is not open work.
    assert_eq!(snapshot.open_count, snapshot.open_items.len() - 1);
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
fn a_confirmed_capture_is_saved_with_its_project_priority_and_date() {
    let workspace = sample_workspace();
    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Revisar acceso de Almendra".into(),
            project_name: Some("andeanvet".into()),
            due_date: Some(TestDay::october(22)),
            priority: Priority::High,
            remind_at: None,
            kind: ItemKind::Task,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: None,
        })
        .unwrap();

    let due_soon = workspace.island_snapshot().unwrap().sections[1].clone();
    let saved = due_soon
        .items
        .iter()
        .find(|item| item.title == "Revisar acceso de Almendra")
        .unwrap();
    assert_eq!(saved.project_name.as_deref(), Some("Andeanvet"));
}

#[test]
fn an_unknown_project_name_is_left_out_instead_of_failing() {
    let workspace = sample_workspace();
    let saved = workspace.save_confirmed(ConfirmedCapture {
        title: "Llamar".into(),
        project_name: Some("Proyecto que no existe".into()),
        due_date: None,
        priority: Priority::Normal,
        remind_at: None,
        kind: ItemKind::Task,
        tags: Vec::new(),
        repeat: tiki_core::domain::Repeat::Never,
        waiting_for: None,
    });
    assert!(saved.is_ok());
}

#[test]
fn the_assistant_sees_open_tasks_without_their_private_notes() {
    let context = sample_workspace().assistant_context().unwrap();

    assert_eq!(context.now.date(), TestDay::today());
    assert_eq!(context.open_tasks.len(), 7);
    let waiting = context
        .open_tasks
        .iter()
        .find(|task| task.status == TaskStatus::Waiting)
        .unwrap();
    assert!(waiting.waiting_days.is_some());

    // Each task's private notes never go to the assistant.
    let tasks_json = serde_json::to_string(&context.open_tasks).unwrap();
    assert!(!tasks_json.contains("notes"));
}

#[test]
fn tags_are_normalized_and_standalone_reminders_list_apart_from_work() {
    let workspace = TimedWorkspace::sample_at(TestDay::today().and_hms_opt(0, 0, 0).unwrap());
    let today = TestDay::today();
    let task_id = workspace
        .save_confirmed(ConfirmedCapture {
            title: "Facturar septiembre".into(),
            project_name: Some("Altamira".into()),
            due_date: Some(TestDay::october(30)),
            priority: Priority::Normal,
            remind_at: None,
            kind: tiki_core::domain::ItemKind::Task,
            tags: vec![
                "#Facturación".into(),
                "facturación".into(),
                " Cierre de mes ".into(),
            ],
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: None,
        })
        .unwrap();
    assert_eq!(
        workspace.task_details(task_id).unwrap().tags,
        ["facturación", "cierre-de-mes"]
    );

    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Llamar a mamá".into(),
            project_name: None,
            due_date: None,
            priority: Priority::Normal,
            remind_at: Some(today.and_hms_opt(17, 0, 0).unwrap()),
            kind: tiki_core::domain::ItemKind::Reminder,
            tags: Vec::new(),
            repeat: tiki_core::domain::Repeat::Never,
            waiting_for: None,
        })
        .unwrap();
    let snapshot = workspace.island_snapshot().unwrap();
    let reminders = snapshot
        .sections
        .iter()
        .find(|section| section.key == "reminders")
        .unwrap();
    assert_eq!(reminders.title, "Avisos");
    assert_eq!(reminders.items[0].title, "Llamar a mamá");
    assert_eq!(reminders.items[0].detail_label, "17:00");
    assert_eq!(snapshot.due_count, 3);

    let alerts = workspace
        .at(today.and_hms_opt(17, 0, 0).unwrap())
        .take_due_task_reminders()
        .unwrap();
    assert_eq!(alerts.len(), 1);
    let after = workspace.island_snapshot().unwrap();
    assert!(
        after
            .sections
            .iter()
            .all(|section| section.key != "reminders")
    );
    assert_eq!(after.delivered_today, 0);
}

#[test]
fn the_assistant_reads_project_notes_and_task_ids() {
    let workspace = sample_workspace();
    let altamira = workspace
        .project_summaries()
        .unwrap()
        .into_iter()
        .find(|project| project.name == "Altamira")
        .unwrap();
    workspace
        .change_project(
            altamira.project_id,
            tiki_core::projects::ProjectChange {
                name: altamira.name.clone(),
                color: altamira.color.clone(),
                notes: "Contacto: Lucía, prefiere correos cortos.".into(),
            },
        )
        .unwrap();

    let context = workspace.assistant_context().unwrap();
    let altamira = context
        .projects
        .iter()
        .find(|project| project.name == "Altamira")
        .unwrap();
    assert_eq!(altamira.notes, "Contacto: Lucía, prefiere correos cortos.");
    assert_eq!(context.projects.len(), 3);
    assert!(context.open_tasks.iter().all(|task| task.task_id.0 > 0));
}

#[test]
fn capture_lessons_keep_only_the_most_recent() {
    let workspace = sample_workspace();
    for index in 0..25 {
        workspace
            .record_capture_lesson(&tiki_core::capture::CaptureLesson {
                sentence: format!("frase {index}"),
                understood: serde_json::json!({ "projectName": null }),
                corrected: serde_json::json!({ "projectName": "Andeanvet" }),
            })
            .unwrap();
    }
    let lessons = workspace.capture_lessons().unwrap();
    assert_eq!(lessons.len(), 20);
    assert_eq!(lessons[0].sentence, "frase 24");
    assert_eq!(lessons[0].corrected["projectName"], "Andeanvet");
}
