mod support;

use chrono::NaiveDateTime;
use support::{TestDay, TimedWorkspace};
use tiki_core::agenda::{AgendaDay, AgendaEntry};
use tiki_core::calendar::{CalendarProvider, Meeting};
use tiki_core::capture::ConfirmedCapture;
use tiki_core::domain::{ItemKind, Priority, Repeat};

fn at(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::october(day).and_hms_opt(hour, minute, 0).unwrap()
}

fn timed(title: &str, kind: ItemKind, remind_at: NaiveDateTime) -> ConfirmedCapture {
    ConfirmedCapture {
        title: title.into(),
        project_name: None,
        due_date: None,
        priority: Priority::Normal,
        remind_at: Some(remind_at),
        kind,
        tags: Vec::new(),
        repeat: Repeat::Never,
        waiting_for: None,
    }
}

fn meeting(id: &str, title: &str, starts_at: NaiveDateTime, all_day: bool) -> Meeting {
    Meeting {
        external_id: id.into(),
        title: title.into(),
        starts_at,
        ends_at: starts_at + chrono::Duration::minutes(30),
        all_day,
        location: String::new(),
        join_url: None,
    }
}

/// Each day's title, with its lines as "kind time title".
fn lines(days: &[AgendaDay]) -> Vec<(String, Vec<String>)> {
    days.iter()
        .map(|day| {
            let entries = day
                .entries
                .iter()
                .map(|entry| match entry {
                    AgendaEntry::Meeting {
                        time_label,
                        meeting,
                    } => format!("meeting {time_label} {}", meeting.title),
                    AgendaEntry::Reminder { time_label, item } => {
                        format!("reminder {time_label} {}", item.title)
                    }
                    AgendaEntry::Task { time_label, item } => {
                        format!("task {time_label} {}", item.title)
                    }
                })
                .collect();
            (day.label.clone(), entries)
        })
        .collect()
}

#[test]
fn the_agenda_lists_meetings_reminders_and_task_alarms_by_day_and_time() {
    let workspace = TimedWorkspace::sample_at(at(21, 8, 0));
    workspace
        .save_confirmed(timed("Tomar agua", ItemKind::Reminder, at(22, 11, 0)))
        .unwrap();
    workspace
        .save_confirmed(timed("Revisar acta", ItemKind::Task, at(21, 15, 0)))
        .unwrap();
    let account = workspace
        .connect_calendar(CalendarProvider::Google, "yo@gmail.com")
        .unwrap();
    workspace
        .store_meetings(
            account,
            &[
                meeting("a", "Daily", at(22, 9, 0), false),
                meeting("b", "Feriado", at(23, 0, 0), true),
            ],
        )
        .unwrap();

    let agenda = workspace.island_snapshot().unwrap().agenda;
    assert_eq!(
        lines(&agenda),
        [
            ("Hoy".to_owned(), vec!["task 15:00 Revisar acta".to_owned()]),
            (
                "Mañana".to_owned(),
                vec![
                    "meeting 09:00–09:30 Daily".to_owned(),
                    "reminder 11:00 Tomar agua".to_owned(),
                ]
            ),
            (
                "viernes 23 de octubre".to_owned(),
                vec!["meeting Todo el día Feriado".to_owned()]
            ),
        ]
    );
}
