mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::briefing::{BriefDetail, BriefMoment, BriefNarrator, DailyBrief, DailyBriefBuilder};
use tiki_core::i18n::{Locale, Translator};

fn sample_brief(settings: &TikiSettings) -> DailyBrief {
    let workspace = SampleWorkspace::create();
    let tasks = workspace.database.tasks().list_open().unwrap();
    DailyBriefBuilder::new(settings).build(&tasks, &workspace.directory(), TestDay::today())
}

#[test]
fn every_open_task_lands_in_one_section() {
    let brief = sample_brief(&TikiSettings::default());

    let titles = |items: &[tiki_core::briefing::BriefItem]| -> Vec<String> {
        items.iter().map(|item| item.title.clone()).collect()
    };
    assert_eq!(
        titles(&brief.overdue),
        ["Acceso solo a análisis de facturas de Almendra"]
    );
    assert_eq!(
        titles(&brief.due_soon),
        [
            "Tracking de formularios comerciales",
            "Mejora en el módulo de Ventas"
        ]
    );
    assert_eq!(
        titles(&brief.your_turn),
        [
            "Saldo vacacional con solicitud rechazada",
            "Completar módulo de importaciones"
        ]
    );
    assert_eq!(brief.waiting.len(), 2);
}

#[test]
fn waiting_past_the_threshold_needs_a_follow_up() {
    let brief = sample_brief(&TikiSettings::default());

    assert_eq!(brief.needs_follow_up.len(), 1);
    assert_eq!(
        brief.needs_follow_up[0].title,
        "Permisos de GitHub para retenciones de IGV"
    );
    assert_eq!(
        brief.needs_follow_up[0].detail,
        BriefDetail::Waiting { days: 3 }
    );
}

#[test]
fn a_wider_window_moves_more_tasks_to_due_soon() {
    let narrow = TikiSettings {
        due_soon_window_days: 1,
        ..TikiSettings::default()
    };
    assert_eq!(sample_brief(&narrow).due_soon.len(), 1);
}

#[test]
fn the_spanish_narration_reads_like_a_morning_brief() {
    let brief = sample_brief(&TikiSettings::default());
    let translator = Translator::new(Locale::Spanish).unwrap();

    let narration = BriefNarrator::new(&translator).narrate(&brief, BriefMoment::Morning);

    assert!(narration.starts_with("Buenos días. Hoy es miércoles 21 de octubre."));
    assert!(narration.contains(
        "Vence pronto: Tracking de formularios comerciales de Altamira y Mejora en el módulo de Ventas de Altamira."
    ));
    assert!(narration.contains("Esperando respuesta:"));
}

#[test]
fn the_english_narration_uses_english_dates_and_lists() {
    let brief = sample_brief(&TikiSettings::default());
    let translator = Translator::new(Locale::English).unwrap();

    let narration = BriefNarrator::new(&translator).narrate(&brief, BriefMoment::Morning);

    assert!(narration.starts_with("Good morning. Today is Wednesday, October 21."));
    assert!(narration.contains(" and "));
}

#[test]
fn the_brief_knows_what_is_due_tomorrow() {
    let brief = sample_brief(&TikiSettings::default());
    let tomorrow: Vec<String> = brief
        .due_tomorrow()
        .into_iter()
        .map(|item| item.title)
        .collect();
    assert_eq!(tomorrow, ["Tracking de formularios comerciales"]);
}

#[test]
fn the_brief_lists_the_reminders_of_one_day() {
    use tiki_core::briefing::BriefItem;
    use tiki_core::domain::{Priority, Repeat, TaskId};
    let reminder = |task_id, at| BriefItem {
        task_id: TaskId(task_id),
        title: format!("Aviso {task_id}"),
        project_id: None,
        project_name: None,
        tags: Vec::new(),
        priority: Priority::Normal,
        detail: BriefDetail::RemindAt {
            at,
            repeat: Repeat::Never,
        },
    };
    let mut brief = DailyBrief::empty(TestDay::today());
    brief.reminders = vec![
        reminder(1, TestDay::today().and_hms_opt(17, 0, 0).unwrap()),
        reminder(2, TestDay::october(22).and_hms_opt(9, 0, 0).unwrap()),
    ];

    let today = brief.reminders_on(TestDay::today());
    assert_eq!(today.len(), 1);
    assert_eq!(today[0].task_id, TaskId(1));
    assert_eq!(
        brief.reminders_on(TestDay::october(22))[0].task_id,
        TaskId(2)
    );
}

#[test]
fn each_brief_line_keeps_the_id_of_its_project() {
    let workspace = SampleWorkspace::create();
    let tasks = workspace.database.tasks().list_open().unwrap();
    let brief = DailyBriefBuilder::new(&TikiSettings::default()).build(
        &tasks,
        &workspace.directory(),
        TestDay::today(),
    );

    assert_eq!(brief.overdue[0].project_id, Some(workspace.andeanvet));
    assert_eq!(brief.your_turn[1].project_id, None);
}

#[test]
fn what_now_starts_with_the_overdue_item_then_today_then_important_work() {
    use tiki_core::briefing::{FocusPicker, FocusReason};
    use tiki_core::domain::Priority;
    let mut brief = sample_brief(&TikiSettings::default());

    let pick = FocusPicker::pick(&brief).unwrap();
    assert_eq!(pick.reason, FocusReason::Overdue);
    assert_eq!(
        pick.item.title,
        "Acceso solo a análisis de facturas de Almendra"
    );

    brief.overdue.clear();
    assert_eq!(
        FocusPicker::pick(&brief).unwrap().reason,
        FocusReason::DueSoon
    );

    brief.your_turn[1].priority = Priority::Urgent;
    let important = FocusPicker::pick(&brief).unwrap();
    assert_eq!(important.reason, FocusReason::Important);
    assert_eq!(important.item.title, "Completar módulo de importaciones");

    brief.due_soon.clear();
    brief.your_turn.clear();
    assert_eq!(
        FocusPicker::pick(&brief).unwrap().reason,
        FocusReason::FollowUp
    );
    brief.needs_follow_up.clear();
    assert!(FocusPicker::pick(&brief).is_none());
}
