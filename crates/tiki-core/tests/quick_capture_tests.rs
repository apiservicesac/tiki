mod support;

use chrono::NaiveTime;
use support::{SampleWorkspace, TestDay};
use tiki_core::TikiSettings;
use tiki_core::capture::QuickCaptureParser;
use tiki_core::i18n::{Locale, Translator};

fn parser(locale: Locale, workspace: &SampleWorkspace) -> QuickCaptureParser {
    let translator = Translator::new(locale).unwrap();
    let clients = workspace.database.clients().list().unwrap();
    QuickCaptureParser::new(
        &translator.catalog().capture,
        &clients,
        TikiSettings::default().default_reminder_time,
    )
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

#[test]
fn a_reminder_with_day_and_time_is_read_and_left_out_of_the_title() {
    let workspace = SampleWorkspace::create();
    let draft = parser(Locale::Spanish, &workspace)
        .parse(
            "revisar acceso andeanvet, avísame el viernes a las 9",
            TestDay::today(),
        )
        .unwrap();

    assert_eq!(draft.title, "Revisar acceso");
    assert_eq!(draft.client_id, Some(workspace.andeanvet));
    assert_eq!(draft.due_date, Some(TestDay::october(23)));
    assert_eq!(
        draft.remind_at,
        Some(TestDay::october(23).and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()))
    );
}

#[test]
fn a_reminder_without_time_uses_the_default_and_an_afternoon_hour_is_understood() {
    let workspace = SampleWorkspace::create();
    let spanish = parser(Locale::Spanish, &workspace);

    let without_time = spanish
        .parse("recuérdame llamar a altamira mañana", TestDay::today())
        .unwrap();
    assert_eq!(without_time.title, "Llamar");
    assert_eq!(
        without_time.remind_at,
        Some(TestDay::october(22).and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()))
    );

    let afternoon = spanish
        .parse("enviar acta a las 3:30", TestDay::today())
        .unwrap();
    assert_eq!(afternoon.title, "Enviar acta");
    assert_eq!(
        afternoon.remind_at,
        Some(TestDay::today().and_time(NaiveTime::from_hms_opt(15, 30, 0).unwrap()))
    );
}

#[test]
fn numbers_in_a_title_are_not_times_and_english_reminders_work() {
    let workspace = SampleWorkspace::create();
    let numbers = parser(Locale::Spanish, &workspace)
        .parse("revisar 3 facturas", TestDay::today())
        .unwrap();
    assert_eq!(numbers.title, "Revisar 3 facturas");
    assert_eq!(numbers.remind_at, None);

    let english = parser(Locale::English, &workspace)
        .parse(
            "call vapestation friday, remind me at 5pm",
            TestDay::today(),
        )
        .unwrap();
    assert_eq!(english.title, "Call");
    assert_eq!(
        english.remind_at,
        Some(TestDay::october(23).and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap()))
    );
}
