//! Test-only CPU spans. No native presentation or allocation-count claims.
use super::*;
use crate::session_mode::integration_tests::{
    Fixture, benchmark_pcm_hashes, display_test_envelope, publish_benchmark_analysis,
    visual_capture,
};
use serde_json::{Value, json};

const BUDGET_US: f64 = 1_000_000.0 / 60.0;

fn summarize(mode: &str, width: f32, height: f32, samples: &[Value], fields: &[&str]) {
    let mut summary = json!({
        "fixture": "eight-native-8khz-varied-v1", "mode": mode,
        "width": width, "height": height, "cold": samples[0],
        "warm_frames": samples.len() - 1, "cpu_budget_us": BUDGET_US,
        "presentation_misses": null, "samples": samples,
    });
    for field in fields {
        let mut values: Vec<_> = samples[1..]
            .iter()
            .map(|row| row[field].as_f64().unwrap())
            .collect();
        values.sort_by(f64::total_cmp);
        summary[format!("{field}_p50_p95_p99")] = json!(
            [0.5, 0.95, 0.99].map(|p| values[((values.len() - 1) as f64 * p).ceil() as usize])
        );
        summary[format!("{field}_cpu_budget_exceedances")] =
            json!(values.iter().filter(|&&value| value > BUDGET_US).count());
    }
    eprintln!("cpu benchmark: {summary}");
}

fn fixture() -> Fixture {
    let fixture = Fixture::new(8);
    for member in 0..8 {
        visual_capture(fixture.engine(), member);
    }
    fixture.engine().label(0, 8000, Some(0)).unwrap();
    fixture.engine().label(16000, 24000, Some(1)).unwrap();
    fixture.engine().label(32000, 40000, Some(129)).unwrap();
    fixture
}

#[test]
#[ignore = "CPU measurement: run serialized with --ignored --test-threads=1"]
fn gain_projection_cpu_measurement() {
    for mode in ["refresh", "cached", "gap-refresh", "dense-refresh", "stale"] {
        let fixture = fixture();
        eprintln!(
            "fixture PCM SHA256 ({mode}): {}",
            json!(benchmark_pcm_hashes(fixture.engine()))
        );
        let envelope = if mode == "dense-refresh" {
            // The existing real EffectiveEnvelope fixture has 32 manual ramps.
            // It changes member zero's first second outside all timed spans.
            let envelope = display_test_envelope(&fixture);
            visual_capture(fixture.engine(), 0);
            envelope
        } else {
            rust_app::GainEnvelope::constant_at_rate(64000, 8000, 0.25).unwrap()
        };
        if mode == "gap-refresh" {
            for member in 0..8 {
                fixture.unverify_display_range(member, 16000, 24000);
            }
        }
        for (width, height) in [(720.0, 560.0), (1000.0, 680.0)] {
            publish_benchmark_analysis(fixture.engine(), envelope.clone());
            let context = egui::Context::default();
            crate::editor::style(&context);
            let mut workspace = SessionUi {
                selection: Some((16000, 24000)),
                playhead: Some(20000),
                ..Default::default()
            };
            let mut samples = Vec::new();
            for frame in 0..41 {
                let query_begin = Instant::now();
                let end = if mode == "dense-refresh" {
                    8000 * 7200
                } else {
                    64000
                };
                let view = fixture.engine().view(Some(0), Some(end), 1000).unwrap();
                let query_us = query_begin.elapsed().as_secs_f64() * 1e6;
                // Force refresh/cached state rather than depend on 33 ms wall-clock expiry.
                let stamp = Instant::now();
                workspace.gain_refresh = if mode == "cached" && frame > 0 {
                    Some(stamp)
                } else {
                    None
                };
                if mode == "stale" && frame == 1 {
                    fixture.engine().cancel_analysis();
                }
                let draw_begin = Instant::now();
                let mut output = context.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            egui::vec2(width, height),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(height - 370.0)
                            .show(ui, |ui| {
                                // Actual clipped-lane query and gain projection are included in draw_us.
                                workspace.waveforms(ui, &view, 564.0, Some(fixture.engine()));
                            });
                    },
                );
                let draw_us = draw_begin.elapsed().as_secs_f64() * 1e6;
                output.textures_delta.clear();
                if mode == "cached" && frame > 0 {
                    assert_eq!(
                        workspace.gain_refresh,
                        Some(stamp),
                        "Cached trial expired; do not classify it as a cache-hit sample"
                    );
                }
                if mode == "stale" && frame > 0 {
                    assert!(workspace.analysis_gain.is_none());
                }
                if mode == "dense-refresh" {
                    assert!(!workspace.analysis_gain.as_ref().unwrap().tracks.is_empty());
                    assert!(
                        workspace
                            .analysis_gain
                            .as_ref()
                            .unwrap()
                            .tracks
                            .iter()
                            .flat_map(|track| &track.columns)
                            .all(|column| column.detail == GainDetail::TooDense)
                    );
                }
                if mode == "gap-refresh" {
                    assert!(!workspace.analysis_gain.as_ref().unwrap().tracks.is_empty());
                    assert!(
                        workspace
                            .analysis_gain
                            .as_ref()
                            .unwrap()
                            .tracks
                            .iter()
                            .flat_map(|track| &track.columns)
                            .all(|column| column.end <= 16000 || column.start >= 24000)
                    );
                }
                let tess_begin = Instant::now();
                let meshes =
                    context.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
                let tessellation_us = tess_begin.elapsed().as_secs_f64() * 1e6;
                let vertices: usize = meshes
                    .iter()
                    .map(|primitive| match &primitive.primitive {
                        egui::epaint::Primitive::Mesh(mesh) => mesh.vertices.len(),
                        _ => 0,
                    })
                    .sum();
                samples.push(json!({"frame":frame, "query_us":query_us,"draw_including_gain_us":draw_us,"tessellation_us":tessellation_us,"total_us":query_us+draw_us+tessellation_us,"vertices":vertices}));
                output.textures_delta.clear();
                std::hint::black_box(meshes);
            }
            summarize(
                mode,
                width,
                height,
                &samples,
                &[
                    "query_us",
                    "draw_including_gain_us",
                    "tessellation_us",
                    "total_us",
                ],
            );
        }
    }
}

#[test]
#[ignore = "CPU measurement: run serialized with --ignored --test-threads=1"]
fn egui_dispatch_to_observed_state_cpu_measurement() {
    let fixture = fixture();
    let view = fixture.engine().view(Some(0), Some(64000), 1000).unwrap();
    for (width, height) in [(720.0, 560.0), (1000.0, 680.0)] {
        let context = egui::Context::default();
        crate::editor::style(&context);
        let mut workspace = SessionUi {
            selection: Some((16000, 24000)),
            ..Default::default()
        };
        let mut samples = Vec::new();
        // Establish actual hover/layout before delivering the first measured wheel event.
        let mut prime = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(width, height))),
                events: vec![egui::Event::PointerMoved(Pos2::new(330.0, 80.0))],
                ..Default::default()
            },
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(height - 370.0)
                    .show(ui, |ui| workspace.waveforms(ui, &view, 564.0, None));
            },
        );
        prime.textures_delta.clear();
        for frame in 0..41 {
            workspace.viewport = None;
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(width, height))),
                events: vec![
                    egui::Event::PointerMoved(Pos2::new(330.0, 80.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, 120.0),
                        modifiers: egui::Modifiers::NONE,
                        source: egui::MouseWheelSource::Unknown,
                        phase: egui::TouchPhase::Move,
                    },
                ],
                ..Default::default()
            };
            let began = Instant::now();
            let mut output = context.run_ui(input, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(height - 370.0)
                    .show(ui, |ui| workspace.waveforms(ui, &view, 564.0, None));
            });
            let observed = workspace.viewport;
            let dispatch_to_observed_state_us = began.elapsed().as_secs_f64() * 1e6;
            let (start, end) =
                observed.expect("Wheel event must update source viewport in this dispatch");
            assert!(end - start < 64000);
            assert_eq!(workspace.selection, Some((16000, 24000)));
            samples.push(json!({"frame":frame,"dispatch_to_observed_state_us":dispatch_to_observed_state_us}));
            output.textures_delta.clear();
        }
        summarize(
            "egui-wheel-dispatch-upper-bound",
            width,
            height,
            &samples,
            &["dispatch_to_observed_state_us"],
        );
    }
}
