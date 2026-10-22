mod support;

use chrono::{NaiveDateTime, NaiveTime};
use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::calendar::{CalendarProvider, Meeting};
use tiki_core::reminders::AlertKind;
use tiki_core::workspace::{FixedClock, TikiWorkspace};

fn at(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::october(day).and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
}

fn workspace_at(now: NaiveDateTime) -> TikiWorkspace {
    TikiWorkspace::new(
        SampleWorkspace::create().database,
        TikiSettings::default(),
        Box::new(FixedClock::at(now)),
    )
    .unwrap()
}

fn meeting(id: &str, title: &str, starts_at: NaiveDateTime, minutes: i64) -> Meeting {
    Meeting {
        external_id: id.into(),
        title: title.into(),
        starts_at,
        ends_at: starts_at + chrono::Duration::minutes(minutes),
        all_day: false,
        location: "Teams".into(),
        join_url: Some("https://teams.microsoft.com/l/meetup".into()),
    }
}

#[test]
fn a_connected_calendar_fills_the_agenda_and_signing_in_again_reuses_the_account() {
    let workspace = workspace_at(at(21, 9, 0));
    let work = workspace
        .connect_calendar(CalendarProvider::Microsoft, "yo@empresa.com")
        .unwrap();
    assert_eq!(
        workspace
            .connect_calendar(CalendarProvider::Microsoft, "YO@empresa.com")
            .unwrap(),
        work
    );
    workspace
        .store_meetings(
            work,
            &[
                meeting("a", "Daily", at(21, 10, 0), 15),
                meeting("b", "CAPEX Claro", at(23, 14, 30), 30),
            ],
        )
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let titles: Vec<&str> = snapshot
        .meetings
        .iter()
        .map(|meeting| meeting.title.as_str())
        .collect();
    assert_eq!(titles, ["Daily", "CAPEX Claro"]);
    assert_eq!(snapshot.meetings[0].account_email, "yo@empresa.com");

    // A meeting removed from the calendar disappears from Tiki at the next sync.
    workspace
        .store_meetings(work, &[meeting("b", "CAPEX Claro", at(23, 14, 30), 30)])
        .unwrap();
    assert_eq!(workspace.island_snapshot().unwrap().meetings.len(), 1);

    workspace.disconnect_calendar(work).unwrap();
    assert_eq!(workspace.island_snapshot().unwrap().meetings.len(), 0);
    assert_eq!(workspace.calendar_accounts().unwrap().len(), 0);
}

#[test]
fn a_meeting_about_to_start_is_announced_once_with_its_link() {
    let workspace = workspace_at(at(21, 9, 50));
    let account = workspace
        .connect_calendar(CalendarProvider::Google, "yo@gmail.com")
        .unwrap();
    workspace
        .store_meetings(account, &[meeting("a", "Daily", at(21, 10, 0), 15)])
        .unwrap();

    assert_eq!(
        workspace.take_due_meeting_alerts(at(21, 9, 40)).unwrap(),
        Vec::new()
    );
    let alerts = workspace.take_due_meeting_alerts(at(21, 9, 50)).unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::Meeting);
    assert_eq!(alerts[0].title, "Reunión en 10 min");
    assert!(alerts[0].body.starts_with("Daily\n10:00–10:15 · Teams"));
    assert_eq!(
        alerts[0].link.as_deref(),
        Some("https://teams.microsoft.com/l/meetup")
    );
    assert_eq!(
        workspace.take_due_meeting_alerts(at(21, 9, 55)).unwrap(),
        Vec::new()
    );

    // Syncing again keeps it announced; moving it makes it new again.
    workspace
        .store_meetings(account, &[meeting("a", "Daily", at(21, 10, 0), 15)])
        .unwrap();
    assert_eq!(
        workspace.take_due_meeting_alerts(at(21, 9, 56)).unwrap(),
        Vec::new()
    );
    workspace
        .store_meetings(account, &[meeting("a", "Daily", at(21, 10, 5), 15)])
        .unwrap();
    assert_eq!(
        workspace
            .take_due_meeting_alerts(at(21, 9, 56))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn now_counts_the_free_time_before_the_next_meeting_and_the_brief_names_meetings() {
    let workspace = workspace_at(at(21, 9, 15));
    let account = workspace
        .connect_calendar(CalendarProvider::Microsoft, "yo@empresa.com")
        .unwrap();
    workspace
        .store_meetings(
            account,
            &[meeting("a", "Revisión con Altamira", at(21, 10, 0), 30)],
        )
        .unwrap();

    let focus = workspace.island_snapshot().unwrap().focus.unwrap();
    assert_eq!(
        focus.reason,
        "Está atrasada · 45 min libres antes de «Revisión con Altamira»"
    );

    let morning = workspace
        .take_due_notification(at(21, 9, 15))
        .unwrap()
        .unwrap();
    assert!(
        morning
            .body
            .ends_with("Reuniones hoy: 1 · la primera a las 10:00")
    );
}
