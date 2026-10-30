mod support;

use chrono::{NaiveDateTime, NaiveTime, TimeDelta};
use support::{TestDay, TimedWorkspace};
use tiki_core::agenda::AgendaEntry;
use tiki_core::alerts::AlertKind;
use tiki_core::calendar::{CalendarProvider, Meeting, MeetingItem, MeetingLink, MeetingPlatform};
use tiki_core::workspace::IslandSnapshot;

const AVATAR: &str = "data:image/jpeg;base64,/9j/4AAQ";

fn at(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::october(day).and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
}

fn workspace_at(now: NaiveDateTime) -> TimedWorkspace {
    TimedWorkspace::sample_at(now)
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

/// The meetings of the Agenda, in its order.
fn agenda_meetings(snapshot: &IslandSnapshot) -> Vec<&MeetingItem> {
    snapshot
        .agenda
        .iter()
        .flat_map(|day| &day.entries)
        .filter_map(|entry| match entry {
            AgendaEntry::Meeting { meeting, .. } => Some(meeting),
            _ => None,
        })
        .collect()
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
                meeting("b", "Comité de inversiones", at(23, 14, 30), 30),
            ],
        )
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let meetings = agenda_meetings(&snapshot);
    let titles: Vec<&str> = meetings
        .iter()
        .map(|meeting| meeting.title.as_str())
        .collect();
    assert_eq!(titles, ["Daily", "Comité de inversiones"]);
    assert_eq!(meetings[0].account.email, "yo@empresa.com");
    assert_eq!(meetings[0].platform, Some(MeetingPlatform::Teams));

    // A meeting removed from the calendar disappears from Tiki at the next sync.
    workspace
        .store_meetings(
            work,
            &[meeting("b", "Comité de inversiones", at(23, 14, 30), 30)],
        )
        .unwrap();
    assert_eq!(
        agenda_meetings(&workspace.island_snapshot().unwrap()).len(),
        1
    );

    workspace.disconnect_calendar(work).unwrap();
    assert_eq!(
        agenda_meetings(&workspace.island_snapshot().unwrap()).len(),
        0
    );
    assert_eq!(workspace.calendar_accounts().unwrap().len(), 0);
}

#[test]
fn an_account_keeps_the_name_and_picture_its_provider_gave() {
    let workspace = workspace_at(at(21, 9, 0));
    let account_id = workspace
        .connect_calendar(CalendarProvider::Google, "ana@acme.com")
        .unwrap();
    let day = TimeDelta::days(1);
    let account = workspace.calendar_account(account_id).unwrap();
    assert_eq!(account.identity.display_name, "");
    assert_eq!(account.identity.avatar, None);
    assert!(account.identity_due(at(21, 9, 0), day));

    workspace
        .update_calendar_identity(account_id, "Ana Pérez", Some(AVATAR))
        .unwrap();
    // Signing in again keeps what Tiki learned.
    workspace
        .connect_calendar(CalendarProvider::Google, "ana@acme.com")
        .unwrap();

    let account = workspace.calendar_account(account_id).unwrap();
    assert_eq!(account.identity.display_name, "Ana Pérez");
    assert_eq!(account.identity.avatar.as_deref(), Some(AVATAR));
    assert!(!account.identity_due(at(22, 8, 59), day));
    assert!(account.identity_due(at(22, 9, 0), day));
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
        workspace
            .at(at(21, 9, 40))
            .take_due_meeting_alerts()
            .unwrap(),
        Vec::new()
    );
    let alerts = workspace
        .at(at(21, 9, 50))
        .take_due_meeting_alerts()
        .unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].kind, AlertKind::Meeting);
    assert_eq!(alerts[0].title, "Reunión en 10 min");
    assert!(
        alerts[0]
            .notification_body()
            .starts_with("Daily\n10:00–10:15 · Teams")
    );
    assert_eq!(alerts[0].summary, "Daily");
    assert_eq!(
        alerts[0].link.as_deref(),
        Some("https://teams.microsoft.com/l/meetup")
    );
    assert_eq!(
        workspace
            .at(at(21, 9, 55))
            .take_due_meeting_alerts()
            .unwrap(),
        Vec::new()
    );

    // Syncing again keeps it announced; moving it makes it new again.
    workspace
        .store_meetings(account, &[meeting("a", "Daily", at(21, 10, 0), 15)])
        .unwrap();
    assert_eq!(
        workspace
            .at(at(21, 9, 56))
            .take_due_meeting_alerts()
            .unwrap(),
        Vec::new()
    );
    workspace
        .store_meetings(account, &[meeting("a", "Daily", at(21, 10, 5), 15)])
        .unwrap();
    assert_eq!(
        workspace
            .at(at(21, 9, 56))
            .take_due_meeting_alerts()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_meeting_alert_names_the_account_and_the_platform_of_its_call() {
    let workspace = workspace_at(at(21, 9, 50));
    let account = workspace
        .connect_calendar(CalendarProvider::Google, "ana@acme.com")
        .unwrap();
    workspace
        .update_calendar_identity(account, "Ana Pérez", Some(AVATAR))
        .unwrap();
    let mut daily = meeting("a", "Daily", at(21, 10, 0), 15);
    daily.join_url = Some("https://meet.google.com/abc-defg-hij".into());
    workspace.store_meetings(account, &[daily]).unwrap();

    let alert = workspace.take_due_meeting_alerts().unwrap().remove(0);
    let origin = alert.origin.as_ref().unwrap();
    assert_eq!(origin.platform, Some(MeetingPlatform::GoogleMeet));
    assert_eq!(origin.account.avatar.as_deref(), Some(AVATAR));
    assert_eq!(origin.label, "Google Meet · Ana Pérez · ana@acme.com");
    assert!(
        alert
            .notification_body()
            .ends_with("\nGoogle Meet · Ana Pérez · ana@acme.com")
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
            &[meeting("a", "Revisión con Acme", at(21, 10, 0), 30)],
        )
        .unwrap();

    let focus = workspace.island_snapshot().unwrap().focus.unwrap();
    assert_eq!(
        focus.reason,
        "Está atrasada · 45 min libres antes de «Revisión con Acme»"
    );

    let morning = workspace
        .at(at(21, 9, 15))
        .take_due_notification()
        .unwrap()
        .unwrap();
    assert!(
        morning
            .notification_body()
            .ends_with("Reuniones hoy: 1 · la primera a las 10:00")
    );
}

#[test]
fn past_meetings_are_not_kept() {
    let workspace = workspace_at(at(21, 9, 0));
    let account = workspace
        .connect_calendar(CalendarProvider::Google, "yo@gmail.com")
        .unwrap();
    workspace
        .store_meetings(
            account,
            &[
                meeting("old", "Ayer", at(20, 10, 0), 30),
                meeting("today", "Hoy", at(21, 10, 0), 30),
            ],
        )
        .unwrap();

    let snapshot = workspace.island_snapshot().unwrap();
    let titles: Vec<&str> = agenda_meetings(&snapshot)
        .iter()
        .map(|meeting| meeting.title.as_str())
        .collect();
    assert_eq!(titles, ["Hoy"]);
}

#[test]
fn the_calendar_window_starts_today_and_covers_the_agenda_days() {
    let workspace = workspace_at(at(21, 15, 30));
    assert_eq!(workspace.calendar_window(), (at(21, 0, 0), at(29, 0, 0)));
}

#[test]
fn the_meetings_of_a_day_are_picked_from_the_agenda() {
    use tiki_core::calendar::{AccountIdentity, CalendarBook, MeetingItem};
    let item = |starts_at| MeetingItem {
        title: "Daily".into(),
        starts_at,
        ends_at: starts_at + chrono::Duration::minutes(15),
        all_day: false,
        location: String::new(),
        join_url: None,
        platform: None,
        account: AccountIdentity {
            provider: CalendarProvider::Microsoft,
            email: "yo@example.com".into(),
            display_name: String::new(),
            avatar: None,
        },
    };
    let agenda = [item(at(21, 10, 0)), item(at(22, 10, 0))];

    let today = CalendarBook::meetings_on(&agenda, TestDay::today());
    assert_eq!(today.len(), 1);
    assert_eq!(today[0].starts_at, at(21, 10, 0));
}

#[test]
fn meetings_starting_soon_come_once_earliest_first_with_their_account() {
    let database = tiki_core::storage::Database::open_in_memory().unwrap();
    let calendar = database.calendar();
    let account = calendar
        .add_account(CalendarProvider::Google, "yo@gmail.com")
        .unwrap();
    calendar
        .update_identity(account, "Yo", Some(AVATAR), at(21, 9, 0))
        .unwrap();
    calendar
        .replace_meetings(
            account,
            at(21, 0, 0),
            at(22, 0, 0),
            &[
                meeting("late", "Segunda", at(21, 10, 10), 30),
                meeting("early", "Primera", at(21, 10, 5), 30),
            ],
        )
        .unwrap();

    let taken = calendar
        .take_starting(at(21, 10, 0), at(21, 10, 15))
        .unwrap();
    let titles: Vec<&str> = taken.iter().map(|meeting| meeting.title.as_str()).collect();
    assert_eq!(titles, ["Primera", "Segunda"]);
    assert_eq!(taken[0].account.email, "yo@gmail.com");
    assert_eq!(taken[0].account.display_name, "Yo");
    assert_eq!(taken[0].account.avatar.as_deref(), Some(AVATAR));
    assert_eq!(
        calendar
            .take_starting(at(21, 10, 0), at(21, 10, 15))
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn a_google_meet_link_opens_with_the_account_the_meeting_comes_from() {
    assert_eq!(
        MeetingLink::for_account("https://meet.google.com/abc-defg-hij", "ana@acme.com"),
        "https://meet.google.com/abc-defg-hij?authuser=ana@acme.com"
    );
}

#[test]
fn other_meeting_links_are_left_as_they_are() {
    let teams = "https://teams.microsoft.com/l/meetup-join/19%3ameeting";
    assert_eq!(MeetingLink::for_account(teams, "ana@acme.com"), teams);
}

#[test]
fn the_platform_of_a_meeting_comes_from_the_host_of_its_link() {
    let platforms = [
        "https://meet.google.com/abc-defg-hij?authuser=ana@acme.com",
        "https://teams.microsoft.com/l/meetup-join/19%3ameeting",
        "https://teams.live.com/meet/9341",
        "https://acme.zoom.us/j/123456",
        "https://acme.webex.com/meet/ana",
    ]
    .map(MeetingPlatform::of);
    assert_eq!(
        platforms,
        [
            MeetingPlatform::GoogleMeet,
            MeetingPlatform::Teams,
            MeetingPlatform::Teams,
            MeetingPlatform::Zoom,
            MeetingPlatform::Other,
        ]
    );
}
