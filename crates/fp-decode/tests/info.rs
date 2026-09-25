#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fp_decode::FileDecoder;

#[test]
fn the_decoder_reports_channels_and_the_frame_count_when_known() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mono.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22_050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..22_050 {
        w.write_sample((i % 100) as i16).unwrap();
    }
    w.finalize().unwrap();
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.channels(), 1);
    assert_eq!(d.frames_hint(), Some(22_050));
    assert_eq!(d.sample_rate(), 22_050);
}
