mod support;

use std::fs::File;

use ogg::reading::PacketReader;
use support::TestFolder;
use tiki_recording::RecordingConfig;
use tiki_recording::media::{MediaSink, OggOpusWriter};

#[test]
fn an_audio_only_session_is_an_ogg_opus_file_that_ends_where_the_audio_ends() {
    let folder = TestFolder::new();
    let path = folder.file(RecordingConfig::AUDIO_FILE);
    let mut writer = Box::new(OggOpusWriter::create(&path).unwrap());
    let one_second_and_a_bit = 48_000 + 500;
    writer
        .write_audio(&vec![0.1; one_second_and_a_bit])
        .unwrap();
    let bytes = writer.finish().unwrap();
    assert_eq!(bytes, std::fs::metadata(&path).unwrap().len());

    let mut packets = PacketReader::new(File::open(&path).unwrap());
    let head = packets.read_packet().unwrap().unwrap();
    assert_eq!(&head.data[..8], b"OpusHead");
    let pre_skip = u64::from(u16::from_le_bytes([head.data[10], head.data[11]]));
    let tags = packets.read_packet().unwrap().unwrap();
    assert_eq!(&tags.data[..8], b"OpusTags");
    let mut last_granule = 0;
    let mut audio_packets = 0;
    while let Some(packet) = packets.read_packet().unwrap() {
        audio_packets += 1;
        last_granule = packet.absgp_page();
    }
    assert_eq!(audio_packets, 51);
    assert_eq!(
        last_granule - pre_skip,
        u64::try_from(one_second_and_a_bit).unwrap()
    );
}
