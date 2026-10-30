mod support;

use std::fmt::Write as _;

use sha2::{Digest, Sha256};
use support::TestFolder;
use support::model_server::{FakeModelServer, ServerBehavior};
use tiki_recording::RecordingError;
use tiki_recording::transcription::{DownloadProgress, ModelDownloader, TranscriptionModel};

const MODEL_BYTES: usize = 300_000;
const CUT_AFTER: usize = 120_000;
const WHOLE_ANSWERS: ServerBehavior = ServerBehavior {
    cut_first_after: None,
    honors_ranges: true,
};

fn model_body() -> Vec<u8> {
    (0..MODEL_BYTES)
        .map(|index| u8::try_from(index % 251).unwrap())
        .collect()
}

fn hex_sha256(bytes: &[u8]) -> &'static str {
    let hex = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        });
    Box::leak(hex.into_boxed_str())
}

fn model_at(server: &FakeModelServer, body: &[u8]) -> TranscriptionModel {
    TranscriptionModel {
        file_name: "ggml-test.bin",
        url: server.url,
        sha256: hex_sha256(body),
        bytes: u64::try_from(body.len()).unwrap(),
    }
}

#[test]
fn a_model_is_downloaded_checked_and_kept_for_next_time() {
    let folder = TestFolder::new();
    let body = model_body();
    let server = FakeModelServer::start(body.clone(), WHOLE_ANSWERS);
    let model = model_at(&server, &body);
    let downloader = ModelDownloader::new(folder.path()).unwrap();
    let mut last = None;

    let path = downloader
        .ensure(&model, &mut |progress| last = Some(progress))
        .unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), body);
    assert_eq!(
        last,
        Some(DownloadProgress {
            downloaded_bytes: model.bytes,
            total_bytes: model.bytes
        })
    );
    downloader.ensure(&model, &mut |_| {}).unwrap();
    assert_eq!(server.requests(), [None]);
}

#[test]
fn a_dropped_download_resumes_where_it_stopped() {
    let folder = TestFolder::new();
    let body = model_body();
    let server = FakeModelServer::start(
        body.clone(),
        ServerBehavior {
            cut_first_after: Some(CUT_AFTER),
            honors_ranges: true,
        },
    );
    let model = model_at(&server, &body);
    let downloader = ModelDownloader::new(folder.path()).unwrap();

    assert!(downloader.ensure(&model, &mut |_| {}).is_err());
    let path = downloader.ensure(&model, &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
    assert_eq!(server.requests(), [None, Some(CUT_AFTER)]);
}

#[test]
fn a_server_that_ignores_the_range_sends_the_whole_model_again() {
    let folder = TestFolder::new();
    let body = model_body();
    let server = FakeModelServer::start(
        body.clone(),
        ServerBehavior {
            cut_first_after: Some(CUT_AFTER),
            honors_ranges: false,
        },
    );
    let model = model_at(&server, &body);
    let downloader = ModelDownloader::new(folder.path()).unwrap();

    assert!(downloader.ensure(&model, &mut |_| {}).is_err());
    let path = downloader.ensure(&model, &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
}

#[test]
fn a_model_that_does_not_match_its_checksum_is_thrown_away() {
    let folder = TestFolder::new();
    let body = model_body();
    let server = FakeModelServer::start(body.clone(), WHOLE_ANSWERS);
    let model = TranscriptionModel {
        sha256: hex_sha256(b"another file"),
        ..model_at(&server, &body)
    };
    let downloader = ModelDownloader::new(folder.path()).unwrap();

    let refused = downloader.ensure(&model, &mut |_| {});

    assert!(matches!(refused, Err(RecordingError::ChecksumMismatch)));
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}
