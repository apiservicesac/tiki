mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::domain::{ExternalReference, Task, TaskId, TaskOrigin, TaskStatus};
use tiki_core::preferences::{TranscriptionQuality, UserPreferences};
use tiki_core::sessions::{RecordingMode, Session};
use tiki_core::storage::Database;
use tiki_core::{TikiError, TikiSettings};

#[test]
fn tasks_survive_a_round_trip() {
    let workspace = SampleWorkspace::create();
    let original = Task::new("Revisar acta de inversiones")
        .unwrap()
        .with_project(workspace.acme)
        .with_due_date(TestDay::october(23))
        .with_notes("Ajustar generación del acta")
        .with_origin(TaskOrigin::External(ExternalReference {
            plugin_id: "odoo".into(),
            connection_id: "acme".into(),
            record_id: "42".into(),
            url: Some("https://acme.example/odoo/helpdesk/42".into()),
        }));

    let task_id = workspace
        .database
        .tasks()
        .insert(&original, TestDay::start())
        .unwrap();
    let stored = workspace.database.tasks().find(task_id).unwrap();

    assert_eq!(stored.id, Some(task_id));
    assert_eq!(stored.title, original.title);
    assert_eq!(stored.notes, original.notes);
    assert_eq!(stored.project_id, original.project_id);
    assert_eq!(stored.due_date, original.due_date);
    assert_eq!(stored.origin, original.origin);
}

#[test]
fn a_task_made_from_a_session_points_back_to_it() {
    let database = Database::open_in_memory().unwrap();
    let session = Session::start("Revisión", RecordingMode::AudioOnly, None, TestDay::start());
    let session_id = database.sessions().insert(&session).unwrap();
    let task = Task::new("Enviar el acta")
        .unwrap()
        .with_origin(TaskOrigin::Session { session_id });

    let task_id = database.tasks().insert(&task, TestDay::start()).unwrap();

    assert_eq!(
        database.tasks().find(task_id).unwrap().origin,
        TaskOrigin::Session { session_id }
    );
}

#[test]
fn updating_a_task_changes_its_status() {
    let workspace = SampleWorkspace::create();
    let task_id = workspace
        .database
        .tasks()
        .insert(&Task::new("Probar").unwrap(), TestDay::start())
        .unwrap();

    let mut task = workspace.database.tasks().find(task_id).unwrap();
    task.move_to(TaskStatus::Done, TestDay::today());
    workspace
        .database
        .tasks()
        .update(&task, TestDay::start())
        .unwrap();

    assert_eq!(
        workspace.database.tasks().find(task_id).unwrap().status,
        TaskStatus::Done
    );
}

#[test]
fn open_tasks_come_sorted_by_due_date_and_exclude_delivered_ones() {
    let workspace = SampleWorkspace::create();
    let open_tasks = workspace.database.tasks().list_open().unwrap();

    assert_eq!(open_tasks.len(), 7);
    assert_eq!(open_tasks[0].due_date, Some(TestDay::october(20)));
    assert!(open_tasks.iter().all(|task| task.status.is_open()));
}

#[test]
fn finding_a_missing_task_reports_not_found() {
    let database = Database::open_in_memory().unwrap();
    assert!(matches!(
        database.tasks().find(TaskId(999)),
        Err(TikiError::NotFound {
            entity: "task",
            id: 999
        })
    ));
}

#[test]
fn projects_are_found_by_name_ignoring_case() {
    let workspace = SampleWorkspace::create();
    let project = workspace.database.projects().find_by_name("atlas").unwrap();
    assert_eq!(
        project.and_then(|project| project.id),
        Some(workspace.atlas)
    );
}

#[test]
fn updating_a_project_that_was_never_saved_reports_it() {
    let database = Database::open_in_memory().unwrap();
    let unsaved = tiki_core::domain::Project::new("Acme", "#2F6BD8");
    assert!(matches!(
        database.projects().update(&unsaved),
        Err(TikiError::UnsavedProject)
    ));
}

#[test]
fn a_failed_transaction_saves_nothing() {
    let database = Database::open_in_memory().unwrap();
    let result: tiki_core::TikiResult<()> = database.transaction(|repositories| {
        repositories
            .tasks()
            .insert(&Task::new("A medias").unwrap(), TestDay::start())?;
        Err(TikiError::EmptyTaskTitle)
    });

    assert!(result.is_err());
    assert_eq!(database.tasks().list_open().unwrap().len(), 0);
}

#[test]
fn a_transaction_saves_every_change_together() {
    let database = Database::open_in_memory().unwrap();
    database
        .transaction(|repositories| {
            repositories
                .tasks()
                .insert(&Task::new("Uno").unwrap(), TestDay::start())?;
            repositories
                .tasks()
                .insert(&Task::new("Dos").unwrap(), TestDay::start())
        })
        .unwrap();

    assert_eq!(database.tasks().list_open().unwrap().len(), 2);
}

#[test]
fn a_task_remembers_when_it_was_created_and_last_changed() {
    let database = Database::open_in_memory().unwrap();
    let nine = TestDay::today().and_hms_opt(9, 0, 0).unwrap();
    let ten = TestDay::today().and_hms_opt(10, 0, 0).unwrap();
    let task_id = database
        .tasks()
        .insert(&Task::new("Probar").unwrap(), nine)
        .unwrap();

    let mut task = database.tasks().find(task_id).unwrap();
    assert_eq!((task.created_at, task.updated_at), (Some(nine), Some(nine)));

    task.title = "Probar de nuevo".into();
    database.tasks().update(&task, ten).unwrap();
    let changed = database.tasks().find(task_id).unwrap();
    assert_eq!(
        (changed.created_at, changed.updated_at),
        (Some(nine), Some(ten))
    );
}

#[test]
fn session_preferences_survive_a_round_trip() {
    let database = Database::open_in_memory().unwrap();
    let defaults = UserPreferences::defaults(&TikiSettings::default());
    assert_eq!(
        defaults.transcription_quality,
        TranscriptionQuality::Balanced
    );
    assert!(defaults.session_notes_automatic);
    assert_eq!(defaults.session_media_days, 90);
    let changed = UserPreferences {
        transcription_quality: TranscriptionQuality::Precise,
        session_notes_automatic: false,
        session_media_days: 30,
        ..defaults.clone()
    };

    database.preferences().save(&changed).unwrap();

    assert_eq!(database.preferences().load(defaults).unwrap(), changed);
}

#[test]
fn turning_off_the_system_stats_survives_a_restart() {
    let sample = SampleWorkspace::create();
    let defaults = UserPreferences::defaults(&TikiSettings::default());
    assert!(defaults.show_system_stats);
    let custom = UserPreferences {
        show_system_stats: false,
        ..defaults.clone()
    };
    sample.database.preferences().save(&custom).unwrap();

    let loaded = sample.database.preferences().load(defaults).unwrap();
    assert!(!loaded.show_system_stats);
}
