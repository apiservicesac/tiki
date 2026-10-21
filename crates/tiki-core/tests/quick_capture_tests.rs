mod support;

use support::{SampleWorkspace, TestDay};
use tiki_core::capture::QuickCaptureParser;
use tiki_core::i18n::{Locale, Translator};

fn parser(locale: Locale, workspace: &SampleWorkspace) -> QuickCaptureParser {
    let translator = Translator::new(locale).unwrap();
    let clients = workspace.database.clients().list().unwrap();
    QuickCaptureParser::new(&translator.catalog().capture, &clients)
}

#[test]
fn understands_client_and_weekday_in_spanish() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::Spanish, &workspace)
        .parse(
            "probar saldo vacacional andeanvet viernes",
            TestDay::today(),
        )
        .unwrap();

    assert_eq!(draft.title, "Probar saldo vacacional");
    assert_eq!(draft.client_id, Some(workspace.andeanvet));
    assert_eq!(draft.due_date, Some(TestDay::october(23)));
}

#[test]
fn accepts_words_without_accents_and_drops_connectors() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::Spanish, &workspace)
        .parse("enviar acta Altamira para manana", TestDay::today())
        .unwrap();

    assert_eq!(draft.title, "Enviar acta");
    assert_eq!(draft.client_id, Some(workspace.altamira));
    assert_eq!(draft.due_date, Some(TestDay::october(22)));
}

#[test]
fn naming_todays_weekday_means_next_week() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::Spanish, &workspace)
        .parse("revisar zonas miércoles", TestDay::today())
        .unwrap();

    assert_eq!(draft.due_date, Some(TestDay::october(28)));
}

#[test]
fn understands_english_too() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::English, &workspace)
        .parse("follow up vapestation by friday", TestDay::today())
        .unwrap();

    assert_eq!(draft.title, "Follow up");
    assert_eq!(draft.client_id, Some(workspace.vapestation));
    assert_eq!(draft.due_date, Some(TestDay::october(23)));
}

#[test]
fn a_sentence_without_dates_or_clients_keeps_every_word() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::Spanish, &workspace)
        .parse("listado de usuarios en las instancias", TestDay::today())
        .unwrap();

    assert_eq!(draft.title, "Listado de usuarios en las instancias");
    assert_eq!(draft.client_id, None);
    assert_eq!(draft.due_date, None);
}
