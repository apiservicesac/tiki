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

/// One Agenda line as the test reads it: what it is, its time and its title.
type Line = (&'static str, String, String);

/// Each day's title with its lines.
fn lines(days: &[AgendaDay]) -> Vec<(String, Vec<Line>)> {
    days.iter()
        .map(|day| {
            let entries = day
                .entries
                .iter()
                .map(|entry| match entry {
                    AgendaEntry::Meeting {
                        time_label,
                        meeting,
                    } => ("meeting", time_label.clone(), meeting.title.clone()),
                    AgendaEntry::Reminder { time_label, item } => {
                        ("reminder", time_label.clone(), item.title.clone())
                    }
                    AgendaEntry::Task { time_label, item } => {
                        ("task", time_label.clone(), item.title.clone())
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
    let line = |kind, time: &str, title: &str| (kind, time.to_owned(), title.to_owned());
    assert_eq!(
        lines(&agenda),
        [
            (
                "Hoy".to_owned(),
                vec![line("task", "15:00", "Revisar acta")]
            ),
            (
                "Mañana".to_owned(),
                vec![
                    line("meeting", "09:00–09:30", "Daily"),
                    line("reminder", "11:00", "Tomar agua"),
                ]
            ),
            (
                "viernes 23 de octubre".to_owned(),
                vec![line("meeting", "Todo el día", "Feriado")]
            ),
        ]
    );
}
