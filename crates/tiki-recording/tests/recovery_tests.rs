mod support;

use std::fs::File;
use std::time::Duration;

use ogg::reading::PacketReader;
use support::TestFolder;
use tiki_recording::RecordingConfig;
use tiki_recording::media::{CrashRecovery, MediaSink, OggOpusWriter, TrackWriter};

const ONE_SECOND: usize = RecordingConfig::MIX_SAMPLE_RATE as usize;
const SPEECH: f32 = 0.5;
const SECONDS_RECORDED: u64 = 6;

/// Tracks written for six seconds, as a recording that Tiki never finished.
fn crashed_tracks(folder: &TestFolder) {
    for file in [
        RecordingConfig::MICROPHONE_TRACK_FILE,
        RecordingConfig::SYSTEM_TRACK_FILE,
    ] {
        let mut track = TrackWriter::create(&folder.file(file)).unwrap();
        for _ in 0..SECONDS_RECORDED {
            track.write(&vec![SPEECH; ONE_SECOND]).unwrap();
        }
        track.finish().unwrap();
    }
}

fn last_granule(path: &std::path::Path) -> u64 {
    let mut packets = PacketReader::new(File::open(path).unwrap());
    let mut last = 0;
    while let Some(packet) = packets.read_packet().unwrap() {
        last = packet.absgp_page();
    }
    last
}

#[test]
fn an_audio_only_session_keeps_its_ogg_opus() {
    let folder = TestFolder::new();
    crashed_tracks(&folder);
    let mut audio =
        Box::new(OggOpusWriter::create(&folder.file(RecordingConfig::AUDIO_FILE)).unwrap());
    audio.write_audio(&vec![SPEECH; ONE_SECOND]).unwrap();
    let bytes = audio.finish().unwrap();

    let recovered = CrashRecovery::recover(folder.path(), true).unwrap();

    assert_eq!(recovered.media, Some((RecordingConfig::AUDIO_FILE, bytes)));
    assert_eq!(recovered.recorded, Duration::from_secs(SECONDS_RECORDED));
}

#[test]
fn a_video_session_loses_its_unfinished_mp4_and_gets_its_audio_back_from_the_tracks() {
    let folder = TestFolder::new();
    crashed_tracks(&folder);
    std::fs::write(folder.file(RecordingConfig::VIDEO_FILE), b"unfinished").unwrap();

    let recovered = CrashRecovery::recover(folder.path(), false).unwrap();

    assert!(!folder.file(RecordingConfig::VIDEO_FILE).exists());
    let audio = folder.file(RecordingConfig::AUDIO_FILE);
    assert_eq!(
        recovered.media.map(|(file, _)| file),
        Some(RecordingConfig::AUDIO_FILE)
    );
    let mut packets = PacketReader::new(File::open(&audio).unwrap());
    let head = packets.read_packet().unwrap().unwrap();
    let pre_skip = u64::from(u16::from_le_bytes([head.data[10], head.data[11]]));
    // Ogg Opus counts 48 kHz samples, whatever the rate of the tracks.
    assert_eq!(last_granule(&audio) - pre_skip, SECONDS_RECORDED * 48_000);
}

#[test]
fn a_session_without_tracks_cannot_be_recovered() {
    let folder = TestFolder::new();
    assert!(CrashRecovery::recover(folder.path(), true).is_err());
}
