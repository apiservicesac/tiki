mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::clients::ClientChange;
use tiki_core::domain::ClientId;
use tiki_core::workspace::{FixedClock, TikiWorkspace};
use tiki_core::{TikiError, TikiSettings};

/// The sample clients' ids, kept after the database moves into the workspace.
struct SampleIds {
    andeanvet: ClientId,
    vapestation: ClientId,
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
fn clients_are_listed_busiest_first_with_their_workload() {
    let (workspace, ids) = sample_workspace();
    let summaries = workspace.client_summaries().unwrap();

    let names: Vec<&str> = summaries
        .iter()
        .map(|client| client.name.as_str())
        .collect();
    assert_eq!(names, ["Altamira", "Andeanvet", "Vapestation"]);
    let andeanvet = summaries
        .iter()
        .find(|client| client.client_id == ids.andeanvet)
        .unwrap();
    assert_eq!(andeanvet.open_count, 3);
    assert_eq!(andeanvet.pressing_count, 1);
}

#[test]
fn a_client_can_be_renamed_and_recolored_but_not_to_a_taken_name() {
    let (workspace, ids) = sample_workspace();
    workspace
        .change_client(
            ids.vapestation,
            ClientChange {
                name: "  Vape Station  ".into(),
                color: "#0E7490".into(),
            },
        )
        .unwrap();
    let renamed = workspace
        .client_summaries()
        .unwrap()
        .into_iter()
        .find(|client| client.client_id == ids.vapestation)
        .unwrap();
    assert_eq!(renamed.name, "Vape Station");
    assert_eq!(renamed.color, "#0E7490");

    let taken = workspace.change_client(
        ids.vapestation,
        ClientChange {
            name: "altamira".into(),
            color: "#0E7490".into(),
        },
    );
    assert!(matches!(taken, Err(TikiError::DuplicateClientName(_))));
    assert!(matches!(
        workspace.add_client("ALTAMIRA"),
        Err(TikiError::DuplicateClientName(_))
    ));
}

#[test]
fn removing_a_client_keeps_its_tasks_without_client() {
    let (workspace, ids) = sample_workspace();
    workspace.remove_client(ids.andeanvet).unwrap();

    assert!(
        workspace
            .client_summaries()
            .unwrap()
            .iter()
            .all(|client| client.client_id != ids.andeanvet)
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
            .all(|item| item.client_name.as_deref() != Some("Andeanvet"))
    );
}
