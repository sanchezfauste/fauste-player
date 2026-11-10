#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Audio changed or not (external editor spec EE6, EE10).

use std::path::PathBuf;

use fp_model::{
    AudioChange, AudioHash, MarkerKind, MarkerSource, MarkersReview, Track, TrackAnalysis, TrackId,
    audio_change,
};

const VERSION: u32 = 3;

fn track() -> Track {
    let mut t = Track::new(TrackId(1), PathBuf::from("/music/Song.flac"));
    t.duration_secs = 240.0;
    t.audio_hash = Some(AudioHash {
        value: 7,
        version: VERSION,
    });
    t
}

fn analysis(hash: Option<u64>, duration: f64) -> TrackAnalysis {
    TrackAnalysis {
        duration_secs: duration,
        cue_out: Some(duration - 1.0),
        segue_start: Some(190.0),
        audio_hash: hash,
        version: VERSION,
        ..TrackAnalysis::default()
    }
}

#[test]
fn same_fingerprint_raises_no_flag() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::SegueStart, Some(100.0));
    t.apply_analysis(&analysis(Some(7), 240.0));
    assert_eq!(t.markers_review, None);
}

#[test]
fn changed_audio_with_a_manual_marker_flags_it() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::SegueStart, Some(100.0));
    t.apply_analysis(&analysis(Some(8), 240.0));
    assert_eq!(t.markers_review, Some(MarkersReview { dropped: vec![] }));
    assert_eq!(
        t.audio_hash,
        Some(AudioHash {
            value: 8,
            version: VERSION
        })
    );
}

#[test]
fn changed_audio_without_manual_markers_raises_no_flag() {
    let mut t = track();
    t.apply_analysis(&analysis(Some(8), 240.0));
    assert_eq!(t.markers_review, None);
}

#[test]
fn no_stored_fingerprint_raises_no_flag_and_stores_one() {
    let mut t = track();
    t.audio_hash = None;
    t.markers.set_manual(MarkerKind::SegueStart, Some(100.0));
    t.apply_analysis(&analysis(Some(8), 240.0));
    assert_eq!(t.markers_review, None);
    assert_eq!(
        t.audio_hash,
        Some(AudioHash {
            value: 8,
            version: VERSION
        })
    );
}

#[test]
fn another_version_raises_no_flag() {
    let mut t = track();
    t.audio_hash = Some(AudioHash {
        value: 7,
        version: VERSION - 1,
    });
    t.markers.set_manual(MarkerKind::SegueStart, Some(100.0));
    t.apply_analysis(&analysis(Some(8), 240.0));
    assert_eq!(t.markers_review, None);
    assert_eq!(
        audio_change(t.audio_hash, None, VERSION),
        AudioChange::Unknown
    );
}

#[test]
fn the_rule_itself() {
    let h = Some(AudioHash {
        value: 1,
        version: 2,
    });
    assert_eq!(audio_change(h, Some(1), 2), AudioChange::Same);
    assert_eq!(audio_change(h, Some(9), 2), AudioChange::Changed);
    assert_eq!(audio_change(h, Some(9), 3), AudioChange::Unknown);
    assert_eq!(audio_change(None, Some(9), 2), AudioChange::Unknown);
}

#[test]
fn a_manual_marker_past_the_new_end_is_dropped_and_replaced_by_the_automatic_one() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(8), 200.0));
    let m = t.markers.get(MarkerKind::SegueStart).unwrap();
    assert_eq!((m.secs, m.source), (190.0, MarkerSource::Auto));
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::SegueStart]
        })
    );
}

#[test]
fn a_manual_marker_inside_the_new_audio_stays() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::SegueStart, Some(150.0));
    t.apply_analysis(&analysis(Some(8), 200.0));
    let m = t.markers.get(MarkerKind::SegueStart).unwrap();
    assert_eq!((m.secs, m.source), (150.0, MarkerSource::Manual));
    assert_eq!(t.markers_review, Some(MarkersReview { dropped: vec![] }));
}

#[test]
fn a_new_flag_keeps_the_union_of_dropped_markers() {
    let mut t = track();
    t.markers_review = Some(MarkersReview {
        dropped: vec![MarkerKind::CueOut],
    });
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(8), 200.0));
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::CueOut, MarkerKind::SegueStart]
        })
    );
}

#[test]
fn a_tag_only_change_keeps_an_existing_flag() {
    let mut t = track();
    t.markers_review = Some(MarkersReview {
        dropped: vec![MarkerKind::CueOut],
    });
    t.markers.set_manual(MarkerKind::SegueStart, Some(100.0));
    t.apply_analysis(&analysis(Some(7), 240.0));
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::CueOut]
        })
    );
}

#[test]
fn the_analysis_stamp_is_stored() {
    use fp_model::FileStamp;
    let mut t = track();
    let mut a = analysis(Some(7), 240.0);
    a.file_stamp = Some(FileStamp {
        size: 10,
        modified_ns: 20,
    });
    t.apply_analysis(&a);
    assert_eq!(t.file_stamp, a.file_stamp);
}

#[test]
fn file_stamp_from_metadata_converts_or_refuses() {
    use fp_model::FileStamp;
    use std::time::{Duration, SystemTime};
    let at = SystemTime::UNIX_EPOCH + Duration::from_nanos(1_500);
    assert_eq!(
        FileStamp::from_metadata(9, at),
        Some(FileStamp {
            size: 9,
            modified_ns: 1_500
        })
    );
    let before = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
    assert_eq!(FileStamp::from_metadata(9, before), None);
}

#[test]
fn an_old_track_document_loads_without_the_new_fields() {
    let t = track();
    let mut v = serde_json::to_value(&t).unwrap();
    let o = v.as_object_mut().unwrap();
    o.remove("file_stamp");
    o.remove("audio_hash");
    o.remove("markers_review");
    let back: Track = serde_json::from_value(v).unwrap();
    assert_eq!(back.file_stamp, None);
    assert_eq!(back.audio_hash, None);
    assert_eq!(back.markers_review, None);
}

/// M1: a dropped manual cue-out takes the inner manual markers with it into
/// the new cue range, as setting the cue-out by hand does.
#[test]
fn a_dropped_cue_out_clamps_the_manual_inner_markers() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::CueOut, Some(230.0));
    // Inside the new audio, but past its automatic cue-out (199 s).
    t.markers.set_manual(MarkerKind::SegueStart, Some(199.5));
    t.markers.set_manual(MarkerKind::OutroStart, Some(199.8));
    t.apply_analysis(&analysis(Some(8), 200.0));
    let out = t.markers.get(MarkerKind::CueOut).unwrap();
    assert_eq!((out.secs, out.source), (199.0, MarkerSource::Auto));
    for kind in [MarkerKind::SegueStart, MarkerKind::OutroStart] {
        let m = t.markers.get(kind).unwrap();
        assert_eq!(
            (m.secs, m.source),
            (199.0, MarkerSource::Manual),
            "{kind:?}"
        );
    }
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::CueOut]
        })
    );
}

/// M2: without a fingerprint to compare (the first pass after updating),
/// a manual marker past the new end is still dropped and named in the
/// flag: a track never reaches air with a marker past its end.
#[test]
fn an_unknown_change_still_drops_a_marker_past_the_new_end() {
    let mut t = track();
    t.audio_hash = None;
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(8), 200.0));
    let m = t.markers.get(MarkerKind::SegueStart).unwrap();
    assert_eq!((m.secs, m.source), (190.0, MarkerSource::Auto));
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::SegueStart]
        })
    );
}

/// M2: the same with a fingerprint from another analysis version.
#[test]
fn another_version_still_drops_a_marker_past_the_new_end() {
    let mut t = track();
    t.audio_hash = Some(AudioHash {
        value: 7,
        version: VERSION - 1,
    });
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(7), 200.0));
    assert_eq!(
        t.markers_review,
        Some(MarkersReview {
            dropped: vec![MarkerKind::SegueStart]
        })
    );
}

/// M2: an unknown change with no length found drops nothing.
#[test]
fn an_unknown_change_without_a_length_drops_nothing() {
    let mut t = track();
    t.audio_hash = None;
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(8), 0.0));
    let m = t.markers.get(MarkerKind::SegueStart).unwrap();
    assert_eq!((m.secs, m.source), (220.0, MarkerSource::Manual));
    assert_eq!(t.markers_review, None);
}

/// M2: the same fingerprint never drops (the length cannot have changed).
#[test]
fn the_same_fingerprint_never_drops() {
    let mut t = track();
    t.markers.set_manual(MarkerKind::SegueStart, Some(220.0));
    t.apply_analysis(&analysis(Some(7), 200.0));
    let m = t.markers.get(MarkerKind::SegueStart).unwrap();
    assert_eq!((m.secs, m.source), (220.0, MarkerSource::Manual));
    assert_eq!(t.markers_review, None);
}
