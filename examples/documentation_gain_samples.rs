//! Sample the real evaluator for the static documentation figure.
use rust_app::{
    AutomaticEnvelope, GainFadeSettings, OWNERSHIP_FRAME_TICKS, SourceTimebase, TICKS_PER_SECOND,
    TemporalEvidence, TemporalEvidenceGrid, TemporalOwnershipCache, TemporalSettings, TrackId,
};

fn main() {
    let frame_ms = OWNERSHIP_FRAME_TICKS * 1000 / TICKS_PER_SECOND;
    assert_eq!(frame_ms, 32);
    let mut cases = Vec::new();
    for (name, end_frame, minimum_gap_ms) in
        [("long", 60, 10), ("short", 32, 10), ("ignored", 32, 768)]
    {
        let rows = (0..100)
            .map(|frame| {
                let value = if (20..end_frame).contains(&frame) {
                    0.01
                } else {
                    0.95
                };
                TemporalEvidence {
                    probability: Some(value),
                    raw_speech: Some(value),
                    assigned_activity: Some(value),
                    preservation_reason: None,
                }
            })
            .collect();
        let evidence = TemporalEvidenceGrid::new(
            vec![TrackId(1)],
            OWNERSHIP_FRAME_TICKS * 100,
            vec![rows],
            vec![0.01; 100],
        )
        .expect("valid illustrative evidence");
        let ownership = TemporalOwnershipCache::default()
            .optimize_evidence(&evidence, TemporalSettings::default(), None)
            .expect("valid illustrative ownership");
        let settings = GainFadeSettings {
            attenuation: 0.25,
            fade_out: TICKS_PER_SECOND * 512 / 1000,
            fade_in: TICKS_PER_SECOND * 256 / 1000,
            minimum_gap: TICKS_PER_SECOND * minimum_gap_ms / 1000,
        };
        let envelope = AutomaticEnvelope::from_ownership(
            &ownership,
            TrackId(1),
            SourceTimebase::new(48000, 153600).unwrap(),
            settings,
            None,
        )
        .expect("valid illustrative envelope");
        let gain = |ms| envelope.gain_at_time(ms * TICKS_PER_SECOND / 1000);
        let end_ms = end_frame as u64 * frame_ms;
        assert_eq!(gain(640), 1.0);
        assert_eq!(gain(end_ms), 1.0);
        let samples: Vec<_> = (0..=3200)
            .step_by(4)
            .map(|ms| {
                let value = gain(ms);
                assert!((0.25..=1.0).contains(&value));
                if ms <= 640 || ms >= end_ms || name == "ignored" {
                    assert_eq!(value, 1.0);
                }
                (ms, value)
            })
            .collect();
        if name != "ignored" {
            let close_ms = if name == "long" { 512 } else { 256 };
            assert_eq!(gain(640 + close_ms), 0.25);
            assert_eq!(gain(640 + close_ms / 2), 0.625);
            assert!((gain(640 + close_ms / 4) - 0.9223633).abs() < 0.000001);
            assert!(gain(641) > gain(640 + close_ms / 2));
        }
        cases.push(serde_json::json!({
            "name": name,
            "gap_start_ms": 640,
            "gap_end_ms": end_ms,
            "minimum_gap_ms": minimum_gap_ms,
            "samples": samples,
        }));
    }
    println!(
        "{}",
        serde_json::json!({
            "source": "AutomaticEnvelope::from_ownership / gain_at_time",
            "sample_rate": 48000,
            "attenuation_gain": 0.25,
            "requested_close_ms": 512,
            "requested_open_ms": 256,
            "cases": cases,
        })
    );
}
