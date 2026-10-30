mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::domain::{ProjectId, TaskStatus};
use tiki_core::projects::ProjectChange;
use tiki_core::workspace::{FixedClock, TikiWorkspace};
use tiki_core::{TikiError, TikiSettings};

/// The sample projects' ids, kept after the database moves into the workspace.
struct SampleIds {
    atlas: ProjectId,
    vertex: ProjectId,
}

fn sample_workspace() -> (TikiWorkspace, SampleIds) {
    let SampleWorkspace {
        database,
        atlas,
        vertex,
        ..
    } = SampleWorkspace::create();
    let workspace = TikiWorkspace::new(
        database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap();
    (workspace, SampleIds { atlas, vertex })
}

#[test]
fn projects_are_listed_busiest_first_with_their_workload() {
    let (workspace, ids) = sample_workspace();
    let summaries = workspace.project_summaries().unwrap();

    let names: Vec<&str> = summaries
        .iter()
        .map(|project| project.name.as_str())
        .collect();
    assert_eq!(names, ["Acme", "Atlas", "Vertex"]);
    let atlas = summaries
        .iter()
        .find(|project| project.project_id == ids.atlas)
        .unwrap();
    assert_eq!(atlas.open_count, 3);
    assert_eq!(atlas.pressing_count, 1);
}

#[test]
fn a_project_can_be_renamed_and_recolored_but_not_to_a_taken_name() {
    let (workspace, ids) = sample_workspace();
    workspace
        .change_project(
            ids.vertex,
            ProjectChange {
                name: "  Vape Station  ".into(),
                color: "#0E7490".into(),
                notes: String::new(),
            },
        )
        .unwrap();
    let renamed = workspace
        .project_summaries()
        .unwrap()
        .into_iter()
        .find(|project| project.project_id == ids.vertex)
        .unwrap();
    assert_eq!(renamed.name, "Vape Station");
    assert_eq!(renamed.color, "#0E7490");

    let taken = workspace.change_project(
        ids.vertex,
        ProjectChange {
            name: "acme".into(),
            color: "#0E7490".into(),
            notes: String::new(),
        },
    );
    assert!(matches!(taken, Err(TikiError::DuplicateProjectName(_))));
    assert!(matches!(
        workspace.add_project("ACME"),
        Err(TikiError::DuplicateProjectName(_))
    ));
}

#[test]
fn removing_a_project_sends_its_items_to_the_trash_without_project() {
    let (workspace, ids) = sample_workspace();
    let in_progress = workspace
        .island_snapshot()
        .unwrap()
        .open_items
        .into_iter()
        .find(|item| item.title == "Saldo vacacional con solicitud rechazada")
        .unwrap()
        .task_id;
    workspace.deliver(in_progress).unwrap();

    workspace.remove_project(ids.atlas).unwrap();

    assert!(
        workspace
            .project_summaries()
            .unwrap()
            .iter()
            .all(|project| project.project_id != ids.atlas)
    );
    assert_eq!(workspace.island_snapshot().unwrap().open_items.len(), 4);
    assert_eq!(workspace.delivered_tasks().unwrap(), Vec::new());
    let trash = workspace.trashed_items().unwrap();
    assert_eq!(trash.len(), 3);
    assert!(trash.iter().all(|item| item.project_name.is_none()));

    workspace.restore_item(in_progress).unwrap();
    let restored = workspace.task_details(in_progress).unwrap();
    assert_eq!(restored.project_name, None);
    assert_eq!(restored.status, TaskStatus::Done);
}

#[test]
fn reminders_do_not_count_as_work_in_a_project() {
    use tiki_core::capture::ConfirmedCapture;
    use tiki_core::domain::{ItemKind, Priority, Repeat};
    let (workspace, _) = sample_workspace();
    workspace
        .save_confirmed(ConfirmedCapture {
            title: "Llamar a Lucía".into(),
            project_name: Some("Acme".into()),
            due_date: None,
            priority: Priority::Normal,
            remind_at: Some(TestDay::today().and_hms_opt(17, 0, 0).unwrap()),
            kind: ItemKind::Reminder,
            tags: Vec::new(),
            repeat: Repeat::Never,
            waiting_for: None,
        })
        .unwrap();

    let acme = workspace
        .project_summaries()
        .unwrap()
        .into_iter()
        .find(|project| project.name == "Acme")
        .unwrap();
    assert_eq!(acme.open_count, 2);
}
