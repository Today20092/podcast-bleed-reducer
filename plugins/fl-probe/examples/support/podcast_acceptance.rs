//! Debug-only format acceptance. No alternate detector, capture or playback path.
use rust_app::{
    AudioSource, EditorAnalysisWorker, EffectiveEnvelopeCache, Project, RenderGeneration, Rounding,
    SourceTimebase, SpeakerId, SupervisedRenderSnapshot, TimeRange,
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU8},
    time::{Duration, Instant},
};

pub type Driver = unsafe extern "C" fn(u32, i64, i64, u32) -> i32;
pub const RATE: u32 = 48000;
// Complete fingerprint blocks isolate processing parity from conservative partial-edge fallback.
pub const FRAMES: usize = 96 * 2048;
pub const LABELS: [(u32, u32, u32); 4] = [
    (0, RATE / 4, 0),
    (RATE / 2, RATE * 3 / 2, 1),
    (RATE * 2, RATE * 11 / 4, 2),
    (RATE * 13 / 4, RATE * 29 / 8, 3),
];

pub unsafe fn query(driver: Driver, op: u32) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let result = unsafe { driver(op, 0, 0, 0) };
        if result >= 0 {
            return result;
        }
        assert!(
            Instant::now() < deadline,
            "Session view remained unavailable after state restore"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn write_wav(path: &Path, samples: &[f32]) {
    let mut file = fs::File::create_new(path).unwrap();
    let length = (samples.len() * 4) as u32;
    file.write_all(b"RIFF").unwrap();
    file.write_all(&(36 + length).to_le_bytes()).unwrap();
    file.write_all(b"WAVEfmt ").unwrap();
    file.write_all(&16_u32.to_le_bytes()).unwrap();
    file.write_all(&3_u16.to_le_bytes()).unwrap();
    file.write_all(&1_u16.to_le_bytes()).unwrap();
    file.write_all(&RATE.to_le_bytes()).unwrap();
    file.write_all(&(RATE * 4).to_le_bytes()).unwrap();
    file.write_all(&4_u16.to_le_bytes()).unwrap();
    file.write_all(&32_u16.to_le_bytes()).unwrap();
    file.write_all(b"data").unwrap();
    file.write_all(&length.to_le_bytes()).unwrap();
    for sample in samples {
        file.write_all(&sample.to_le_bytes()).unwrap();
    }
}

fn read_float_mono(path: &Path) -> Vec<f32> {
    let bytes = fs::read(path).unwrap();
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    let mut offset = 12;
    let mut format = false;
    while offset + 8 <= bytes.len() {
        let length = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let data = &bytes[offset + 8..offset + 8 + length];
        match &bytes[offset..offset + 4] {
            b"fmt " => {
                assert_eq!(u16::from_le_bytes(data[..2].try_into().unwrap()), 3);
                assert_eq!(u16::from_le_bytes(data[2..4].try_into().unwrap()), 1);
                assert_eq!(u16::from_le_bytes(data[14..16].try_into().unwrap()), 32);
                format = true;
            }
            b"data" => {
                assert!(format);
                assert_eq!(length, FRAMES * 4);
                return data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|sample| f32::from_le_bytes(*sample))
                    .collect();
            }
            _ => {}
        }
        offset += 8 + length + length % 2;
    }
    panic!("Rendered float WAV has no audio");
}

pub struct Fixture {
    pub source: [Vec<f32>; 2],
    pub rendered: [Vec<f32>; 2],
    pub root: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "cleanslate-podcast-format-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let mut random = 7654321_u32;
        let noise = (0..FRAMES.div_ceil(3))
            .map(|_| {
                random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                ((random >> 8) as f32 / 16777216.0 - 0.5) * 0.2
            })
            .collect::<Vec<_>>();
        let mut source: [Vec<f32>; 2] = std::array::from_fn(|speaker| {
            (0..FRAMES)
                .map(|frame| {
                    let index = frame / 3;
                    let a = noise[index]
                        * if (8000..24000).contains(&index) || (48000..60000).contains(&index) {
                            1.0
                        } else {
                            0.0001
                        };
                    let b = noise[(index + 700) % noise.len()]
                        * if (32000..44000).contains(&index) || (52000..60000).contains(&index) {
                            1.0
                        } else {
                            0.0001
                        };
                    if speaker == 0 {
                        a + b * 0.1
                    } else {
                        b + a * 0.2
                    }
                })
                .collect()
        });
        for (member, samples) in source.iter_mut().enumerate() {
            samples[0] = 0.2 + member as f32 * 0.1;
            samples[FRAMES - 1] = -0.25 - member as f32 * 0.1;
        }
        let paths: Vec<_> = (0..2)
            .map(|member| {
                let path = root.join(format!("Mic{}.wav", member + 1));
                write_wav(&path, &source[member]);
                path
            })
            .collect();
        let mut project = Project::create_from_sources(
            root.join("oracle.cleanslate"),
            paths
                .iter()
                .map(|path| AudioSource::mono(path.clone()))
                .collect(),
        )
        .unwrap();
        project
            .set_parameters(rust_app::AutomixParameters {
                attenuation_db: -12.0,
                open_ms: 15.0,
                close_ms: 100.0,
                ..Default::default()
            })
            .unwrap();
        project.set_supplied_alignment(true).unwrap();
        let grid = SourceTimebase::new(RATE, FRAMES as u64).unwrap();
        for (start, end, mask) in LABELS {
            let range = TimeRange::new(
                grid.time_at_frame(u64::from(start), Rounding::Floor)
                    .unwrap(),
                grid.time_at_frame(u64::from(end), Rounding::Ceil).unwrap(),
            )
            .unwrap();
            project
                .add_reference_label(
                    range,
                    (0..2)
                        .filter(|&i| mask & (1 << i) != 0)
                        .map(|i| SpeakerId(i + 1))
                        .collect(),
                    &[],
                )
                .unwrap();
        }
        let progress = AtomicU8::new(0);
        let cancel = AtomicBool::new(false);
        let analysis = EditorAnalysisWorker::default()
            .analyze(&project, &root.join("prepared"), &progress, &cancel)
            .unwrap();
        let effective = analysis
            .install(&mut project, &mut EffectiveEnvelopeCache::default(), None)
            .unwrap();
        // Known reference truth: absent microphone attenuates; the labeled speaker stays unity.
        assert!(effective.envelopes[1].gain_at(RATE as usize) < 0.9);
        assert_eq!(effective.envelopes[0].gain_at(RATE as usize), 1.0);
        let snapshot = SupervisedRenderSnapshot::capture(&project, &analysis, &effective).unwrap();
        project.save().unwrap();
        let generation = RenderGeneration::default();
        rust_app::prepare_supervised_float_render(
            &project,
            &analysis,
            &effective,
            &snapshot,
            &root.join("render"),
            &progress,
            None,
        )
        .unwrap()
        .publish(&project, &generation, generation.current(), None)
        .unwrap();
        let rendered = std::array::from_fn(|member| {
            read_float_mono(&root.join(format!("render/{:02}-Mic{}.wav", member + 1, member + 1)))
        });
        Self {
            source,
            rendered,
            root,
        }
    }
}

/// configure(offline, audition), reset(), process(member, source-time, mono PCM).
pub unsafe fn run(
    driver: Driver,
    mut configure: impl FnMut(bool, u32),
    mut reset: impl FnMut(),
    mut process: impl FnMut(usize, i64, &[f32]) -> Vec<f32>,
    latency: usize,
) {
    unsafe {
        assert_eq!(driver(u32::MAX, 0, 0, 0), -1);
        let fixture = Fixture::new();
        configure(false, 0);
        for member in 0..2 {
            process(member, 0, &fixture.source[member][..1024]);
        }
        assert_eq!(driver(0, 0, 0, 0), 0);
        for offset in (0..FRAMES).step_by(1024) {
            for member in 0..2 {
                process(
                    member,
                    offset as i64,
                    &fixture.source[member][offset..(offset + 1024).min(FRAMES)],
                );
            }
            assert_eq!(driver(0, 0, 0, 0), 0);
        }
        for (start, end, mask) in LABELS {
            assert_eq!(driver(1, i64::from(start), i64::from(end), mask), 0);
        }
        assert_eq!(driver(1, 0, i64::MAX, 256), -1);
        assert_eq!(driver(2, 0, 0, 0), 0);
        let deadline = Instant::now() + Duration::from_secs(30);
        while driver(3, 0, 0, 0) != 2 {
            assert!(
                Instant::now() < deadline,
                "Format Session Analyze did not complete"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        for offline in [false, true] {
            for audition in [0, 1, 2] {
                configure(offline, audition);
                reset();
                let mut outputs: [Vec<f32>; 2] = std::array::from_fn(|_| Vec::new());
                for offset in (0..FRAMES + latency).step_by(1024) {
                    let frames = 1024.min(FRAMES + latency - offset);
                    for (member, output) in outputs.iter_mut().enumerate() {
                        let input = (offset..offset + frames)
                            .map(|n| fixture.source[member].get(n).copied().unwrap_or(0.0))
                            .collect::<Vec<_>>();
                        output.extend(process(member, offset as i64, &input));
                    }
                    assert_eq!(driver(0, 0, 0, 0), 0);
                }
                for (member, samples) in outputs.iter().enumerate() {
                    assert!(samples[..latency].iter().all(|&sample| sample == 0.0));
                    let output = &samples[latency..latency + FRAMES];
                    for (n, actual) in output.iter().enumerate() {
                        let expected = match audition {
                            0 => fixture.source[member][n],
                            1 => fixture.rendered[member][n],
                            _ => fixture.source[member][n] - fixture.rendered[member][n],
                        };
                        assert!(
                            (*actual - expected).abs() < 2e-6,
                            "format PCM mismatch: offline={offline} mode={audition} member={member} sample={n} actual={actual} expected={expected}"
                        );
                    }
                    assert_eq!(output.len(), FRAMES);
                }
            }
        }
        configure(false, 1);
        reset();
        let mut changed_output: [Vec<f32>; 2] = std::array::from_fn(|_| Vec::new());
        for offset in (0..FRAMES + latency).step_by(1024) {
            let frames = 1024.min(FRAMES + latency - offset);
            for (member, output) in changed_output.iter_mut().enumerate() {
                let input = (offset..offset + frames)
                    .map(|n| {
                        fixture.source[member].get(n).copied().unwrap_or(0.0)
                            * if member == 1 { -1.0 } else { 1.0 }
                    })
                    .collect::<Vec<_>>();
                output.extend(process(member, offset as i64, &input));
            }
            assert_eq!(driver(0, 0, 0, 0), 0);
        }
        for (member, output) in changed_output.iter().enumerate() {
            for (n, sample) in output[latency..latency + FRAMES].iter().enumerate() {
                let expected = fixture.source[member][n] * if member == 1 { -1.0 } else { 1.0 };
                assert_eq!(
                    *sample, expected,
                    "changed source must preserve original at source sample {n}"
                );
            }
        }
        assert_eq!(
            driver(3, 0, 0, 0),
            0,
            "changed PCM invalidates current analysis"
        );
        assert_eq!(
            driver(2, 0, 0, 0),
            -1,
            "changed saved examples require review"
        );
        println!(
            "PASS actual-format Session: native capture + explicit maintained Analyze; realtime/offline Original/Processed/Removed match independently rendered float WAV, including first/last sample and fixed PDC. Oracle files retained at {}",
            fixture.root.display()
        );
    }
}
