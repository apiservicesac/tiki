use std::time::Duration;

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
}
