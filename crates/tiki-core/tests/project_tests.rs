mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::domain::ProjectId;
use tiki_core::projects::ProjectChange;
use tiki_core::workspace::{FixedClock, TikiWorkspace};
use tiki_core::{TikiError, TikiSettings};

/// The sample projects' ids, kept after the database moves into the workspace.
struct SampleIds {
    andeanvet: ProjectId,
    vapestation: ProjectId,
}

fn sample_workspace() -> (TikiWorkspace, SampleIds) {
    let SampleWorkspace {
        database,
        andeanvet,
        vapestation,
        ..
    } = SampleWorkspace::create();
    let workspace = TikiWorkspace::new(
        database,
        TikiSettings::default(),
        Box::new(FixedClock::new(TestDay::today())),
    )
    .unwrap();
    (
        workspace,
        SampleIds {
            andeanvet,
            vapestation,
        },
    )
}

#[test]
fn projects_are_listed_busiest_first_with_their_workload() {
    let (workspace, ids) = sample_workspace();
    let summaries = workspace.project_summaries().unwrap();

    let names: Vec<&str> = summaries
        .iter()
        .map(|project| project.name.as_str())
        .collect();
    assert_eq!(names, ["Altamira", "Andeanvet", "Vapestation"]);
    let andeanvet = summaries
        .iter()
        .find(|project| project.project_id == ids.andeanvet)
        .unwrap();
    assert_eq!(andeanvet.open_count, 3);
    assert_eq!(andeanvet.pressing_count, 1);
}

#[test]
fn a_project_can_be_renamed_and_recolored_but_not_to_a_taken_name() {
    let (workspace, ids) = sample_workspace();
    workspace
        .change_project(
            ids.vapestation,
            ProjectChange {
                name: "  Vape Station  ".into(),
                color: "#0E7490".into(),
            },
        )
        .unwrap();
    let renamed = workspace
        .project_summaries()
        .unwrap()
        .into_iter()
        .find(|project| project.project_id == ids.vapestation)
        .unwrap();
    assert_eq!(renamed.name, "Vape Station");
    assert_eq!(renamed.color, "#0E7490");

    let taken = workspace.change_project(
        ids.vapestation,
        ProjectChange {
            name: "altamira".into(),
            color: "#0E7490".into(),
        },
    );
    assert!(matches!(taken, Err(TikiError::DuplicateProjectName(_))));
    assert!(matches!(
        workspace.add_project("ALTAMIRA"),
        Err(TikiError::DuplicateProjectName(_))
    ));
}

#[test]
fn removing_a_project_keeps_its_tasks_without_project() {
    let (workspace, ids) = sample_workspace();
    workspace.remove_project(ids.andeanvet).unwrap();

    assert!(
        workspace
            .project_summaries()
            .unwrap()
            .iter()
            .all(|project| project.project_id != ids.andeanvet)
    );
    let snapshot = workspace.island_snapshot().unwrap();
    let all_items: Vec<_> = snapshot
        .sections
        .iter()
        .flat_map(|section| &section.items)
        .collect();
    assert_eq!(all_items.len(), 7);
    assert!(
        all_items
            .iter()
            .all(|item| item.project_name.as_deref() != Some("Andeanvet"))
    );
}
