use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};

use wasapi::{
    AudioCaptureClient, AudioClient, DeviceEnumerator, Direction, Handle, SampleType, StreamMode,
    WaveFormat, initialize_mta,
};

use super::AudioSource;
use crate::error::{RecordingError, RecordingResult};
use crate::recording_config::RecordingConfig;

/// Which Windows audio endpoint a source captures.
#[derive(Debug, Clone, Copy)]
pub enum WasapiEndpoint {
    /// The default microphone: the user.
    Microphone,
    /// A loopback of the default speakers or headphones: everyone else in the call.
    SystemAudio,
}

impl WasapiEndpoint {
    fn name(self) -> &'static str {
        match self {
            WasapiEndpoint::Microphone => "microphone",
            WasapiEndpoint::SystemAudio => "system audio",
        }
    }

    /// A loopback capture opens the render device and captures from it.
    fn device_direction(self) -> Direction {
        match self {
            WasapiEndpoint::Microphone => Direction::Capture,
            WasapiEndpoint::SystemAudio => Direction::Render,
        }
    }
}

/// What the capture thread sends: captured samples, or why it stopped.
type CapturedAudio = Result<Vec<f32>, String>;

/// One Windows audio endpoint captured with WASAPI in shared, event-driven mode. Windows
/// converts the device's format to 48 kHz mono float for Tiki. The COM objects stay on the
/// capture thread; only samples cross to the recorder.
pub struct WasapiSource {
    endpoint: WasapiEndpoint,
    captured: Receiver<CapturedAudio>,
    running: Arc<AtomicBool>,
    capture: Option<JoinHandle<()>>,
}

impl WasapiSource {
    const BITS_PER_SAMPLE: usize = 32;
    const MONO: usize = 1;
    /// How long the thread waits for audio before it looks whether to stop. A loopback
    /// stream sends nothing while nothing plays, so waiting longer is normal.
    const EVENT_WAIT_MS: u32 = 200;

    /// Starts capturing; fails when the endpoint cannot be opened.
    pub fn start(endpoint: WasapiEndpoint) -> RecordingResult<Self> {
        let (sender, captured) = mpsc::channel();
        let (started_sender, started) = mpsc::sync_channel(1);
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = Arc::clone(&running);
        let capture = thread::Builder::new()
            .name(endpoint.name().to_owned())
            .spawn(move || Self::capture(endpoint, &thread_running, &sender, &started_sender))?;
        let opened = started.recv().unwrap_or_else(|_| Err(String::new()));
        let source = Self {
            endpoint,
            captured,
            running,
            capture: Some(capture),
        };
        opened.map_err(|reason| source.failure(reason))?;
        Ok(source)
    }

    fn capture(
        endpoint: WasapiEndpoint,
        running: &AtomicBool,
        sender: &Sender<CapturedAudio>,
        started: &SyncSender<Result<(), String>>,
    ) {
        let opened = Self::open(endpoint);
        let (client, capture_client, event) = match opened {
            Ok(opened) => {
                let _ = started.send(Ok(()));
                opened
            }
            Err(error) => {
                let _ = started.send(Err(error.to_string()));
                return;
            }
        };
        while running.load(Ordering::Acquire) {
            // A timeout only means silence: loop to look whether to stop.
            if event.wait_for_event(Self::EVENT_WAIT_MS).is_err() {
                continue;
            }
            match Self::read_packets(&capture_client) {
                Ok(samples) if samples.is_empty() => {}
                Ok(samples) => {
                    if sender.send(Ok(samples)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(error.to_string()));
                    break;
                }
            }
        }
        let _ = client.stop_stream();
    }

    fn open(
        endpoint: WasapiEndpoint,
    ) -> Result<(AudioClient, AudioCaptureClient, Handle), Box<dyn Error>> {
        initialize_mta().ok()?;
        let device = DeviceEnumerator::new()?.get_default_device(&endpoint.device_direction())?;
        let mut client = device.get_iaudioclient()?;
        let format = WaveFormat::new(
            Self::BITS_PER_SAMPLE,
            Self::BITS_PER_SAMPLE,
            &SampleType::Float,
            usize::try_from(RecordingConfig::MIX_SAMPLE_RATE)?,
            Self::MONO,
            None,
        );
        let (default_period, _minimum_period) = client.get_device_period()?;
        client.initialize_client(
            &format,
            &Direction::Capture,
            &StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: default_period,
            },
        )?;
        let event = client.set_get_eventhandle()?;
        let capture_client = client.get_audiocaptureclient()?;
        client.start_stream()?;
        Ok((client, capture_client, event))
    }

    /// Every packet waiting in the device, as samples; packets Windows marks silent become
    /// zeros.
    fn read_packets(capture_client: &AudioCaptureClient) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut bytes = VecDeque::new();
        while capture_client.get_next_packet_size()?.unwrap_or_default() > 0 {
            let before = bytes.len();
            let info = capture_client.read_from_device_to_deque(&mut bytes)?;
            if info.flags.silent {
                bytes.range_mut(before..).for_each(|byte| *byte = 0);
            }
        }
        let bytes: Vec<u8> = bytes.into();
        // Each 32-bit float sample is four little-endian bytes.
        let (samples, _incomplete) = bytes.as_chunks();
        Ok(samples.iter().copied().map(f32::from_le_bytes).collect())
    }

    fn failure(&self, reason: String) -> RecordingError {
        RecordingError::Capture {
            source_name: self.endpoint.name(),
            reason,
        }
    }
}

impl AudioSource for WasapiSource {
    fn name(&self) -> &'static str {
        self.endpoint.name()
    }

    fn drain_into(&mut self, samples: &mut Vec<f32>) -> RecordingResult<()> {
        loop {
            match self.captured.try_recv() {
                Ok(Ok(captured)) => samples.extend(captured),
                Ok(Err(reason)) => return Err(self.failure(reason)),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return Ok(()),
            }
        }
    }
}

impl Drop for WasapiSource {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(capture) = self.capture.take() {
            let _ = capture.join();
        }
    }
}
