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
