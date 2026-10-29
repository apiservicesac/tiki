use tiki_core::sessions::VideoQuality;
use tiki_recording::video::VideoPreset;

const SECONDS_PER_HOUR: u64 = 3_600;
const BITS_PER_BYTE: u64 = 8;
const BYTES_PER_GIGABYTE: u64 = 1_000_000_000;

fn bytes_per_hour(preset: VideoPreset) -> u64 {
    u64::from(preset.bitrate) * SECONDS_PER_HOUR / BITS_PER_BYTE
}

#[test]
fn an_hour_of_normal_video_stays_under_a_gigabyte() {
    let normal = VideoPreset::for_quality(VideoQuality::Normal);
    assert_eq!(normal.frame_rate, 15);
    assert!(bytes_per_hour(normal) < BYTES_PER_GIGABYTE);
}

#[test]
fn high_video_is_smoother_and_sharper() {
    let normal = VideoPreset::for_quality(VideoQuality::Normal);
    let high = VideoPreset::for_quality(VideoQuality::High);
    assert!(high.frame_rate > normal.frame_rate);
    assert!(high.bitrate > normal.bitrate);
}
