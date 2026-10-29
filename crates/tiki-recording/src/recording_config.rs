use std::time::Duration;

use crate::transcription::TranscriptionModel;

/// Fixed values of recording and transcription, kept in one place.
pub struct RecordingConfig;

impl RecordingConfig {
    /// Both sources are captured, aligned and mixed at this rate, mono.
    pub const MIX_SAMPLE_RATE: u32 = 48_000;
    /// Whisper reads 16 kHz mono; each source is also written at this rate.
    pub const TRANSCRIPTION_SAMPLE_RATE: u32 = 16_000;
    /// How often the recorder moves audio from the sources to the files.
    pub const PUMP_INTERVAL: Duration = Duration::from_millis(20);
    /// Audio a source may hold beyond what the clock expects before the surplus is dropped:
    /// two sound cards never tick at exactly the same rate.
    pub const MAX_SOURCE_BACKLOG: Duration = Duration::from_millis(200);
    /// The WAV tracks rewrite their header this often, so a crash loses at most this much.
    pub const TRACK_FLUSH_INTERVAL: Duration = Duration::from_secs(5);
    /// How loud each source goes into the mix (1.0 keeps it as captured).
    pub const MICROPHONE_GAIN: f32 = 1.0;
    pub const SYSTEM_GAIN: f32 = 1.0;
    /// Opus for audio-only sessions: 20 ms frames at about 48 kbit/s (~22 MB per hour).
    pub const OPUS_FRAME: Duration = Duration::from_millis(20);
    pub const OPUS_BITRATE: i32 = 48_000;
    /// The largest Opus packet the encoder may write for one frame.
    pub const OPUS_MAX_PACKET_BYTES: usize = 4_000;
    /// An Ogg page is closed after this many packets, so a crash keeps everything but the
    /// last second.
    pub const OPUS_PACKETS_PER_PAGE: u32 = 50;
    pub const OPUS_VENDOR: &'static str = "Tiki";
    /// The low-pass filter in front of the 48 kHz → 16 kHz decimation: its length and its
    /// cut-off, just below the new Nyquist frequency.
    pub const DECIMATION_TAPS: usize = 63;
    pub const DECIMATION_CUTOFF_HZ: f32 = 7_200.0;

    /// Files inside a session's folder: each source for the transcription, and the media
    /// played back.
    pub const MICROPHONE_TRACK_FILE: &'static str = "microphone.wav";
    pub const SYSTEM_TRACK_FILE: &'static str = "system.wav";
    pub const AUDIO_FILE: &'static str = "session.ogg";
    pub const VIDEO_FILE: &'static str = "session.mp4";

    /// Whisper models, downloaded once from whisper.cpp's repository and checked against
    /// their SHA-256: fast, the balanced default, and precise.
    pub const FAST_MODEL: TranscriptionModel = TranscriptionModel {
        file_name: "ggml-base-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin",
        sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
        bytes: 59_707_625,
    };
    pub const BALANCED_MODEL: TranscriptionModel = TranscriptionModel {
        file_name: "ggml-small-q5_1.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin",
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        bytes: 190_085_487,
    };
    pub const PRECISE_MODEL: TranscriptionModel = TranscriptionModel {
        file_name: "ggml-large-v3-turbo-q5_0.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin",
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        bytes: 574_041_195,
    };
    /// A download still in progress is kept with this extension, so it can resume.
    pub const PARTIAL_DOWNLOAD_EXTENSION: &'static str = "part";
    /// How long the model server may take to accept the connection.
    pub const DOWNLOAD_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
    /// Bytes read from the network or the disk at a time.
    pub const DOWNLOAD_BUFFER_BYTES: usize = 64 * 1024;
    /// Whisper uses at most this share of the processor's cores, so the computer stays
    /// responsive while it transcribes.
    pub const TRANSCRIPTION_CORE_DIVISOR: usize = 2;
    /// Whisper's segments this likely to be silence are dropped: on quiet audio it tends to
    /// invent words.
    pub const NO_SPEECH_PROBABILITY_LIMIT: f32 = 0.6;
    /// A track whose loudest sample stays below this is silent and is not transcribed.
    pub const SILENT_TRACK_PEAK: f32 = 0.01;
    /// Whisper timestamps count hundredths of a second.
    pub const WHISPER_TIMESTAMP_MS: i64 = 10;
    /// Whisper detects the language of each track.
    pub const WHISPER_AUTO_LANGUAGE: &'static str = "auto";
}
