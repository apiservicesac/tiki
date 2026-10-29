mod support;

use chrono::{Duration, NaiveDateTime, NaiveTime};
use support::{TestDay, TimedWorkspace};
use tiki_core::TikiError;
use tiki_core::activity::ItemEvent;
use tiki_core::alerts::AlertKind;
use tiki_core::calendar::{CalendarProvider, Meeting};
use tiki_core::i18n::{Locale, Translator};
use tiki_core::sessions::{
    MeetingLink, ProposedSessionTask, RecordingMode, RetentionPolicy, SessionExport, SessionId,
    SessionNotes, SessionStatus, Speaker, Transcript, TranscriptSegment,
};
use tiki_core::storage::Database;

fn at(hour: u32, minute: u32) -> NaiveDateTime {
    TestDay::today().and_time(NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time"))
}

fn workspace() -> TimedWorkspace {
    TimedWorkspace::open(Database::open_in_memory().expect("database"), at(10, 0))
}

fn line(speaker: Speaker, start_ms: i64, text: &str) -> TranscriptSegment {
    TranscriptSegment {
        speaker,
        start_ms,
        end_ms: start_ms + 1_000,
        text: text.to_owned(),
    }
}

/// A session recorded from 10:00 to 10:30 and transcribed with `lines`.
fn transcribed(workspace: &TimedWorkspace, lines: Vec<TranscriptSegment>) -> SessionId {
    let session = workspace
        .at(at(10, 0))
        .sessions()
        .start(RecordingMode::AudioOnly)
        .unwrap();
    let session_id = session.id.unwrap();
    let sessions = workspace.at(at(10, 30)).sessions();
    sessions
        .stop(session_id, "session.ogg".to_owned(), 1_000)
        .unwrap();
    sessions.begin_next_transcription().unwrap();
    sessions
        .save_transcript(
            session_id,
            &Transcript::merge([lines]),
            Some("es".to_owned()),
        )
        .unwrap();
    session_id
}

#[test]
fn a_session_without_a_meeting_is_titled_by_its_start() {
    let workspace = workspace();
    let session = workspace
        .sessions()
        .start(RecordingMode::AudioOnly)
        .unwrap();
    assert_eq!(session.title, "Sesión del Mié 21, 10:00");
    assert_eq!(session.status, SessionStatus::Recording);
    assert_eq!(session.folder_name().unwrap(), "2026-10-21-1");
}

#[test]
fn a_session_of_a_meeting_takes_its_title() {
    let workspace = workspace();
    let account = workspace
        .connect_calendar(CalendarProvider::Google, "ana@example.com")
        .unwrap();
    let meeting = Meeting {
        external_id: "review".to_owned(),
        title: "Revisión con Andeanvet".to_owned(),
        starts_at: at(9, 55),
        ends_at: at(10, 30),
        all_day: false,
        location: String::new(),
        join_url: None,
    };
    workspace.store_meetings(account, &[meeting]).unwrap();
    let session = workspace
        .sessions()
        .start(RecordingMode::AudioAndVideo)
        .unwrap();
    let stored = workspace.sessions().find(session.id.unwrap()).unwrap();
    assert_eq!(stored.title, "Revisión con Andeanvet");
    assert_eq!(
        stored.meeting,
        Some(MeetingLink {
            title: "Revisión con Andeanvet".to_owned(),
            starts_at: at(9, 55),
        })
    );
    assert_eq!(stored.mode, RecordingMode::AudioAndVideo);
}

#[test]
fn only_one_session_records_at_a_time() {
    let workspace = workspace();
    let sessions = workspace.sessions();
    sessions.start(RecordingMode::AudioOnly).unwrap();
    assert!(matches!(
        sessions.start(RecordingMode::AudioOnly),
        Err(TikiError::AlreadyRecording)
    ));
}

#[test]
fn a_session_that_could_not_start_recording_is_forgotten() {
    let workspace = workspace();
    let sessions = workspace.sessions();
    let session_id = sessions
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    sessions.abandon(session_id).unwrap();
    assert_eq!(sessions.list().unwrap().len(), 0);
    assert!(sessions.start(RecordingMode::AudioOnly).is_ok());
}

#[test]
fn the_recorded_time_leaves_pauses_out() {
    let workspace = workspace();
    let session_id = workspace
        .sessions()
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    workspace
        .at(at(10, 10))
        .sessions()
        .pause(session_id)
        .unwrap();
    let paused = workspace
        .at(at(10, 15))
        .sessions()
        .find(session_id)
        .unwrap();
    assert_eq!(paused.recorded(at(10, 15)), Duration::minutes(10));
    workspace
        .at(at(10, 20))
        .sessions()
        .resume(session_id)
        .unwrap();
    let stopped = workspace
        .at(at(10, 30))
        .sessions()
        .stop(session_id, "session.ogg".to_owned(), 42)
        .unwrap();
    assert_eq!(stopped.recorded(at(23, 0)), Duration::minutes(20));
    assert_eq!(stopped.status, SessionStatus::Queued);
}

#[test]
fn a_session_cut_by_a_crash_ends_where_its_tracks_end_and_is_transcribed() {
    let workspace = workspace();
    let session_id = workspace
        .sessions()
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    workspace
        .at(at(10, 10))
        .sessions()
        .pause(session_id)
        .unwrap();
    workspace
        .at(at(10, 20))
        .sessions()
        .resume(session_id)
        .unwrap();
    workspace
        .at(at(10, 25))
        .sessions()
        .pause(session_id)
        .unwrap();

    let recovered = workspace
        .at(at(18, 0))
        .sessions()
        .recover(
            session_id,
            Duration::minutes(15),
            Some(("session.ogg".to_owned(), 500)),
        )
        .unwrap();

    assert_eq!(recovered.status, SessionStatus::Queued);
    assert_eq!(recovered.ended_at, Some(at(10, 25)));
    assert_eq!(recovered.recorded(at(23, 0)), Duration::minutes(15));
    assert!(recovered.has_media());
    assert!(
        workspace
            .sessions()
            .begin_next_transcription()
            .unwrap()
            .is_some()
    );
}

#[test]
fn a_video_session_does_not_pause() {
    let workspace = workspace();
    let session_id = workspace
        .sessions()
        .start(RecordingMode::AudioAndVideo)
        .unwrap()
        .id
        .unwrap();
    assert!(matches!(
        workspace.sessions().pause(session_id),
        Err(TikiError::PauseNeedsAudioOnly)
    ));
}

#[test]
fn a_session_moves_only_along_its_life_cycle() {
    let workspace = workspace();
    let session_id = transcribed(&workspace, Vec::new());
    assert!(matches!(
        workspace.sessions().pause(session_id),
        Err(TikiError::SessionTransition {
            from: "ready",
            to: "paused"
        })
    ));
}

#[test]
fn the_transcript_joins_both_sources_by_time() {
    let transcript = Transcript::merge([
        vec![
            line(Speaker::You, 0, " Hola "),
            line(Speaker::You, 5_000, "¿Listo?"),
        ],
        vec![
            line(Speaker::Others, 2_000, "Buenas"),
            line(Speaker::Others, 6_000, "  "),
        ],
    ]);
    let order: Vec<(Speaker, &str)> = transcript
        .segments()
        .iter()
        .map(|segment| (segment.speaker, segment.text.as_str()))
        .collect();
    assert_eq!(
        order,
        [
            (Speaker::You, "Hola"),
            (Speaker::Others, "Buenas"),
            (Speaker::You, "¿Listo?")
        ]
    );
}

#[test]
fn the_model_is_needed_from_the_first_recording_until_the_queue_is_empty() {
    let workspace = workspace();
    let sessions = workspace.sessions();
    assert!(!sessions.awaits_transcription().unwrap());
    let session_id = sessions
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    assert!(sessions.awaits_transcription().unwrap());
    sessions
        .stop(session_id, "session.ogg".to_owned(), 1)
        .unwrap();
    assert!(sessions.awaits_transcription().unwrap());
    sessions.begin_next_transcription().unwrap();
    sessions
        .save_transcript(session_id, &Transcript::default(), None)
        .unwrap();
    assert!(!sessions.awaits_transcription().unwrap());
}

#[test]
fn a_transcribed_session_is_ready_and_leaves_the_queue() {
    let workspace = workspace();
    let session_id = transcribed(&workspace, vec![line(Speaker::Others, 0, "Empezamos")]);
    let details = workspace.sessions().details(session_id).unwrap();
    assert_eq!(details.summary.status, SessionStatus::Ready);
    assert_eq!(details.language.as_deref(), Some("es"));
    assert_eq!(details.summary.started_label, "Mié 21 · 10:00");
    assert_eq!(details.transcript.segments().len(), 1);
    assert!(
        workspace
            .sessions()
            .begin_next_transcription()
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_failing_transcription_is_retried_then_waits_for_the_user() {
    let workspace = workspace();
    let sessions = workspace.sessions();
    let session_id = sessions
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    sessions
        .stop(session_id, "session.ogg".to_owned(), 1)
        .unwrap();
    for _ in 1..workspace.settings().transcription_attempts {
        sessions.begin_next_transcription().unwrap().unwrap();
        let retried = sessions
            .fail_transcription(session_id, "no model".to_owned())
            .unwrap();
        assert_eq!(retried.status, SessionStatus::Queued);
    }
    sessions.begin_next_transcription().unwrap().unwrap();
    let failed = sessions
        .fail_transcription(session_id, "no model".to_owned())
        .unwrap();
    assert_eq!(failed.status, SessionStatus::Failed);
    assert!(sessions.begin_next_transcription().unwrap().is_none());

    sessions.retry_transcription(session_id).unwrap();
    assert!(sessions.begin_next_transcription().unwrap().is_some());
}

#[test]
fn a_transcription_cut_short_goes_back_to_the_queue() {
    let workspace = workspace();
    let sessions = workspace.sessions();
    let session_id = sessions
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    sessions
        .stop(session_id, "session.ogg".to_owned(), 1)
        .unwrap();
    sessions.begin_next_transcription().unwrap();
    sessions.requeue_interrupted_transcriptions().unwrap();
    assert_eq!(
        sessions.find(session_id).unwrap().status,
        SessionStatus::Queued
    );
    assert!(sessions.begin_next_transcription().unwrap().is_some());
}

#[test]
fn search_finds_words_without_accents_or_case_and_by_prefix() {
    let workspace = workspace();
    let session_id = transcribed(
        &workspace,
        vec![
            line(Speaker::Others, 0, "Revisemos la facturación de octubre"),
            line(Speaker::You, 4_000, "Mañana envío el informe"),
        ],
    );
    let hits = workspace.sessions().search("FACTURACION oct").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].session_id, session_id);
    assert_eq!(hits[0].speaker, Speaker::Others);
    assert_eq!(hits[0].start_ms, 0);
    assert_eq!(
        workspace.sessions().search("inform").unwrap()[0].start_ms,
        4_000
    );
    assert_eq!(workspace.sessions().search("\"(*").unwrap().len(), 0);
    assert_eq!(workspace.sessions().search("   ").unwrap().len(), 0);
}

#[test]
fn a_deleted_session_leaves_the_search() {
    let workspace = workspace();
    let session_id = transcribed(&workspace, vec![line(Speaker::You, 0, "presupuesto anual")]);
    workspace.sessions().delete(session_id).unwrap();
    assert_eq!(workspace.sessions().search("presupuesto").unwrap().len(), 0);
    assert_eq!(workspace.sessions().list().unwrap().len(), 0);
}

#[test]
fn a_proposed_task_becomes_a_task_of_the_session_once() {
    let workspace = workspace();
    let session_id = transcribed(&workspace, vec![line(Speaker::You, 0, "Envío el acta")]);
    let notes = SessionNotes {
        summary: "Se revisó el acta.".to_owned(),
        key_points: vec!["Acta lista".to_owned()],
        decisions: vec!["Enviarla hoy".to_owned()],
    };
    let proposed = [ProposedSessionTask {
        title: "Enviar el acta".to_owned(),
        due_date: Some(TestDay::october(22)),
    }];
    let sessions = workspace.sessions();
    sessions.save_notes(session_id, &notes, &proposed).unwrap();
    let item = sessions.details(session_id).unwrap().action_items[0].clone();

    let task_id = sessions.apply_action_item(item.action_item_id).unwrap();
    assert_eq!(
        sessions.apply_action_item(item.action_item_id).unwrap(),
        task_id
    );

    let task = workspace.task_details(task_id).unwrap();
    assert_eq!(task.title, "Enviar el acta");
    assert_eq!(task.due_date, Some(TestDay::october(22)));
    assert_eq!(
        workspace.activity_of(task_id).unwrap()[0].event,
        ItemEvent::Created
    );
    let details = sessions.details(session_id).unwrap();
    assert_eq!(details.notes, Some(notes.clone()));
    assert_eq!(details.action_items[0].task_id, Some(task_id));

    // Writing the notes again keeps what the user already applied.
    sessions.save_notes(session_id, &notes, &proposed).unwrap();
    let action_items = sessions.details(session_id).unwrap().action_items;
    assert_eq!(action_items.len(), 2);
    assert_eq!(action_items[0].task_id, Some(task_id));
}

#[test]
fn old_media_expires_but_the_transcript_stays() {
    let workspace = workspace();
    let session_id = transcribed(&workspace, vec![line(Speaker::You, 0, "Hola")]);
    let policy = RetentionPolicy {
        media_days: workspace.settings().session_media_days,
    };
    let later = at(10, 0) + Duration::days(91);
    assert_eq!(
        workspace
            .at(at(11, 0))
            .sessions()
            .expired_media(policy)
            .unwrap()
            .len(),
        0
    );
    let expired = workspace
        .at(later)
        .sessions()
        .expired_media(policy)
        .unwrap();
    assert_eq!(expired.len(), 1);
    assert_eq!(workspace.sessions().media_bytes().unwrap(), 1_000);

    workspace.sessions().media_removed(session_id).unwrap();
    assert_eq!(workspace.sessions().expired_media(policy).unwrap().len(), 0);
    assert_eq!(workspace.sessions().media_bytes().unwrap(), 0);
    let details = workspace.sessions().details(session_id).unwrap();
    assert!(!details.summary.has_media);
    assert_eq!(details.transcript.segments().len(), 1);
}

#[test]
fn a_call_brings_a_quiet_offer_to_record_with_the_consent_reminder() {
    let mut workspace = workspace();
    assert_eq!(workspace.take_recording_suggestion().unwrap(), None);
    workspace.set_in_call(true);
    let offer = workspace.take_recording_suggestion().unwrap().unwrap();
    assert_eq!(offer.alert.kind, AlertKind::Recording);
    assert_eq!(offer.alert.title, "¿Grabar esta llamada?");
    assert_eq!(offer.alert.tips, ["Si grabas, avisa a los participantes."]);
    assert!(!offer.delivery.sound);
    assert_eq!(workspace.take_recording_suggestion().unwrap(), None);
}

#[test]
fn the_snapshot_shows_the_session_being_recorded() {
    let workspace = workspace();
    assert_eq!(workspace.island_snapshot().unwrap().recording, None);
    let session_id = workspace
        .sessions()
        .start(RecordingMode::AudioOnly)
        .unwrap()
        .id
        .unwrap();
    let recording = workspace
        .at(at(10, 5))
        .island_snapshot()
        .unwrap()
        .recording
        .unwrap();
    assert_eq!(recording.session_id, session_id);
    assert_eq!(recording.recorded_ms, 5 * 60 * 1_000);
    assert!(!recording.paused);
}

#[test]
fn a_session_exports_as_subtitles_text_and_markdown_notes() {
    let workspace = workspace();
    let session_id = transcribed(
        &workspace,
        vec![
            line(Speaker::Others, 0, "Revisemos la facturación"),
            line(Speaker::You, 65_250, "Mañana envío el informe"),
        ],
    );
    let notes = SessionNotes {
        summary: "Se revisó la facturación.".to_owned(),
        key_points: vec!["Octubre cerrado".to_owned()],
        decisions: Vec::new(),
    };
    let proposed = [ProposedSessionTask {
        title: "Enviar el informe".to_owned(),
        due_date: None,
    }];
    workspace
        .sessions()
        .save_notes(session_id, &notes, &proposed)
        .unwrap();
    let details = workspace.sessions().details(session_id).unwrap();
    let translator = Translator::new(Locale::Spanish).unwrap();
    let export = SessionExport::new(&translator);

    assert_eq!(
        export.folder_name(&details),
        "Tiki · 2026-10-21 · Sesión del Mié 21, 10-00"
    );
    assert_eq!(
        export.subtitles(&details.transcript),
        "1\n00:00:00,000 --> 00:00:01,000\nOtros: Revisemos la facturación\n\n\
         2\n00:01:05,250 --> 00:01:06,250\nTú: Mañana envío el informe\n"
    );
    assert_eq!(
        export.text(&details.transcript),
        "[00:00:00] Otros: Revisemos la facturación\n[00:01:05] Tú: Mañana envío el informe"
    );
    assert_eq!(
        export.notes(&details).unwrap(),
        "# Sesión del Mié 21, 10:00\n\nMié 21 · 10:00\n\n## Resumen\n\nSe revisó la facturación.\n\n\
         ## Puntos clave\n\n- Octubre cerrado\n\n## Tareas propuestas\n\n- [ ] Enviar el informe\n"
    );
}
