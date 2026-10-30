use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use ogg::writing::{PacketWriteEndInfo, PacketWriter};
use opus::{Application, Bitrate, Channels, Encoder};

use super::MediaSink;
use crate::error::RecordingResult;
use crate::recording_config::RecordingConfig;

/// The mix of an audio-only session as Ogg Opus (RFC 7845): small (about 22 MB per hour)
/// and played by the interface's web view. Pages close every second, so a crash keeps
/// what came before.
pub struct OggOpusWriter {
    path: PathBuf,
    packets: PacketWriter<'static, BufWriter<File>>,
    encoder: Encoder,
    /// Samples waiting to fill the next 20 ms frame.
    pending: Vec<f32>,
    frame_samples: usize,
    /// Decoder delay the player skips, at 48 kHz.
    pre_skip: u16,
    /// Ogg Opus counts time at 48 kHz whatever the input rate: granules per input sample.
    granules_per_sample: u64,
    input_rate: u32,
    /// Samples encoded so far, not counting the padding of the last frame.
    encoded_samples: u64,
    packets_in_page: u32,
}

impl OggOpusWriter {
    /// Any stream number is valid; one file holds one stream.
    const STREAM_SERIAL: u32 = 1;
    const HEAD_MAGIC: &'static [u8] = b"OpusHead";
    const TAGS_MAGIC: &'static [u8] = b"OpusTags";
    const HEAD_VERSION: u8 = 1;
    const MONO: u8 = 1;
    /// Mapping family 0: one or two channels, no mapping table.
    const CHANNEL_MAPPING: u8 = 0;
    const OUTPUT_GAIN: i16 = 0;
    const NO_USER_COMMENTS: u32 = 0;
    /// Ogg Opus granule positions always count 48 kHz samples (RFC 7845).
    const GRANULE_RATE: u32 = 48_000;
    const MILLIS_PER_SECOND: u128 = 1_000;

    /// A writer for the mix, at `RecordingConfig::MIX_SAMPLE_RATE`.
    pub fn create(path: &Path) -> RecordingResult<Self> {
        Self::create_at(path, RecordingConfig::MIX_SAMPLE_RATE)
    }

    /// A writer for mono audio at `input_rate`, one of the rates Opus takes (48, 24, 16,
    /// 12 or 8 kHz): the tracks rebuild a session's audio at 16 kHz after a crash.
    pub fn create_at(path: &Path, input_rate: u32) -> RecordingResult<Self> {
        let mut encoder = Encoder::new(input_rate, Channels::Mono, Application::Voip)?;
        encoder.set_bitrate(Bitrate::Bits(RecordingConfig::OPUS_BITRATE))?;
        let granules_per_sample = u64::from(Self::GRANULE_RATE / input_rate);
        let lookahead = u64::try_from(encoder.get_lookahead()?).unwrap_or_default();
        let pre_skip = u16::try_from(lookahead * granules_per_sample).unwrap_or_default();
        let frame_samples = usize::try_from(
            u128::from(input_rate) * RecordingConfig::OPUS_FRAME.as_millis()
                / Self::MILLIS_PER_SECOND,
        )
        .unwrap_or_default();
        let mut writer = Self {
            path: path.to_path_buf(),
            packets: PacketWriter::new(BufWriter::new(File::create(path)?)),
            encoder,
            pending: Vec::with_capacity(frame_samples),
            frame_samples,
            pre_skip,
            granules_per_sample,
            input_rate,
            encoded_samples: 0,
            packets_in_page: 0,
        };
        writer.write_headers()?;
        Ok(writer)
    }

    /// The identification and comment headers, each on its own page.
    fn write_headers(&mut self) -> RecordingResult<()> {
        let mut head = Vec::from(Self::HEAD_MAGIC);
        head.extend([Self::HEAD_VERSION, Self::MONO]);
        head.extend(self.pre_skip.to_le_bytes());
        head.extend(self.input_rate.to_le_bytes());
        head.extend(Self::OUTPUT_GAIN.to_le_bytes());
        head.push(Self::CHANNEL_MAPPING);
        self.packets
            .write_packet(head, Self::STREAM_SERIAL, PacketWriteEndInfo::EndPage, 0)?;

        let vendor = RecordingConfig::OPUS_VENDOR.as_bytes();
        let mut tags = Vec::from(Self::TAGS_MAGIC);
        tags.extend(
            u32::try_from(vendor.len())
                .unwrap_or_default()
                .to_le_bytes(),
        );
        tags.extend(vendor);
        tags.extend(Self::NO_USER_COMMENTS.to_le_bytes());
        self.packets
            .write_packet(tags, Self::STREAM_SERIAL, PacketWriteEndInfo::EndPage, 0)?;
        Ok(())
    }

    /// Encodes one full frame; the last packet of the stream ends it.
    fn encode_frame(
        &mut self,
        frame: &[f32],
        real_samples: usize,
        last: bool,
    ) -> RecordingResult<()> {
        let mut packet = vec![0; RecordingConfig::OPUS_MAX_PACKET_BYTES];
        let length = self.encoder.encode_float(frame, &mut packet)?;
        packet.truncate(length);
        self.encoded_samples += u64::try_from(real_samples).unwrap_or_default();
        self.packets_in_page += 1;
        let end = if last {
            PacketWriteEndInfo::EndStream
        } else if self.packets_in_page >= RecordingConfig::OPUS_PACKETS_PER_PAGE {
            self.packets_in_page = 0;
            PacketWriteEndInfo::EndPage
        } else {
            PacketWriteEndInfo::NormalPacket
        };
        let granule = u64::from(self.pre_skip) + self.encoded_samples * self.granules_per_sample;
        self.packets
            .write_packet(packet, Self::STREAM_SERIAL, end, granule)?;
        Ok(())
    }
}

impl MediaSink for OggOpusWriter {
    fn write_audio(&mut self, mixed: &[f32]) -> RecordingResult<()> {
        self.pending.extend_from_slice(mixed);
        while self.pending.len() >= self.frame_samples {
            let frame: Vec<f32> = self.pending.drain(..self.frame_samples).collect();
            self.encode_frame(&frame, self.frame_samples, false)?;
        }
        Ok(())
    }

    /// Pads the last frame with silence (the granule position tells the player where the
    /// real audio ends) and closes the stream.
    fn finish(mut self: Box<Self>) -> RecordingResult<u64> {
        let real_samples = self.pending.len();
        let mut frame = std::mem::take(&mut self.pending);
        frame.resize(self.frame_samples, 0.0);
        self.encode_frame(&frame, real_samples, true)?;
        let mut file = self.packets.into_inner();
        file.flush()?;
        Ok(std::fs::metadata(&self.path)?.len())
    }
}
