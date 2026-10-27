use cleanslate_gate_detector::{Detector, MODEL_SHA256, RECIPE};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicU64, Ordering},
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    time::Instant,
};

struct CountAlloc;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
// This bounded experiment uses reviewed reference IDs/bindings, not arbitrary project annotations.
const REFERENCE_PROJECT_SHA256: &str =
    "cb1180b1e8bcbeb1d136d6480a2227549407fad75f66769d8f81270bf2680d78";
const OVERLAP_PROJECT_SHA256: &str =
    "20bb912825964a2ba9b717525477f6af365abb3e434084cf7836e6b5725cbd98";
const PUBLIC_CATEGORIES: &str = include_str!("../fixtures/public-categories-v1.json");
type Case = (String, Vec<f32>, serde_json::Value);
unsafe impl GlobalAlloc for CountAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountAlloc = CountAlloc;

fn excerpt(path: &Path, start: f64, duration: f64) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    if &header[..4] != b"RIFF" || &header[8..] != b"WAVE" {
        return Err("not WAVE".into());
    }
    let mut format_ok = false;
    loop {
        let mut chunk = [0; 8];
        file.read_exact(&mut chunk)?;
        let size = u32::from_le_bytes(chunk[4..].try_into()?) as u64;
        let body = file.stream_position()?;
        if &chunk[..4] == b"fmt " {
            let mut fmt = [0; 16];
            file.read_exact(&mut fmt)?;
            format_ok = u16::from_le_bytes(fmt[..2].try_into()?) == 3
                && u16::from_le_bytes(fmt[2..4].try_into()?) == 1
                && u32::from_le_bytes(fmt[4..8].try_into()?) == 48_000
                && u16::from_le_bytes(fmt[14..16].try_into()?) == 32;
        }
        if &chunk[..4] == b"data" {
            if !format_ok {
                return Err("expected mono48k float32 source".into());
            }
            let offset = (start * 48_000.0).round() as u64 * 4;
            let length = (duration * 48_000.0).round() as usize * 4;
            if offset + length as u64 > size {
                return Err("excerpt outside source".into());
            }
            file.seek(SeekFrom::Start(body + offset))?;
            let mut bytes = vec![0; length];
            file.read_exact(&mut bytes)?;
            return Ok(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| f32::from_le_bytes(*b))
                .collect());
        }
        file.seek(SeekFrom::Start(body + size + size % 2))?;
    }
}

fn file_sha256(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65_536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn public_categories(root: &Path) -> Result<Vec<Case>, Box<dyn std::error::Error>> {
    let manifest: serde_json::Value = serde_json::from_str(PUBLIC_CATEGORIES)?;
    let mut cases = Vec::new();
    for fixture in manifest["fixtures"]
        .as_array()
        .ok_or("missing public fixtures")?
    {
        let expected = fixture["bytes"].as_u64().ok_or("missing fixture length")?;
        let path = root.join(fixture["file"].as_str().ok_or("missing fixture file")?);
        let mut bytes = Vec::with_capacity(expected as usize + 1);
        File::open(path)?
            .take(expected + 1)
            .read_to_end(&mut bytes)?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if bytes.len() as u64 != expected
            || bytes.len() % 4 != 0
            || fixture["sha256"].as_str() != Some(&hash)
        {
            return Err("public fixture PCM identity mismatch".into());
        }
        let samples: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        if samples.iter().any(|s| !s.is_finite()) {
            return Err("public fixture contains nonfinite PCM".into());
        }
        cases.push((fixture["case"].as_str().ok_or("missing fixture case")?.into(), samples,
            json!({"fixture":fixture,"manifest_sha256":format!("{:x}",Sha256::digest(PUBLIC_CATEGORIES.as_bytes())),"provenance_level":manifest["provenance_level"],"selection":manifest["selection"],"conversion":manifest["conversion"],"limits":manifest["limits"]})));
    }
    Ok(cases)
}

fn recorded_overlap(root: &Path) -> Result<Case, Box<dyn std::error::Error>> {
    let root = root.join("quick-test-10min");
    let project_bytes = std::fs::read(root.join("261005-quick-test.cleanslate"))?;
    if format!("{:x}", Sha256::digest(&project_bytes)) != OVERLAP_PROJECT_SHA256 {
        return Err("reviewed overlap project identity mismatch".into());
    }
    let project: serde_json::Value = serde_json::from_slice(&project_bytes)?;
    let mut verified = Vec::new();
    for source in project["sources"]
        .as_array()
        .ok_or("missing overlap sources")?
    {
        let track = source["track_id"].as_u64().ok_or("missing overlap track")?;
        let hash = file_sha256(&root.join(format!("261005-channel-{track}-10min.wav")))?;
        if source["fingerprint"].as_str() != Some(&hash) {
            return Err("overlap source binding mismatch".into());
        }
        verified.push(json!({"track":track,"sha256":hash}));
    }
    let reference = project["references"]
        .as_array()
        .ok_or("missing overlap references")?
        .iter()
        .find(|r| r["id"] == 7)
        .ok_or("overlap reference7 missing")?;
    if reference["active_speakers"] != json!([1, 2]) {
        return Err("overlap reference speaker set changed".into());
    }
    for binding in reference["sources"]
        .as_array()
        .ok_or("missing overlap binding")?
    {
        if !verified
            .iter()
            .any(|s| s["track"] == binding["track"] && s["sha256"] == binding["fingerprint"])
        {
            return Err("overlap reference source binding mismatch".into());
        }
    }
    let ticks = project["ticks_per_second"]
        .as_f64()
        .ok_or("missing overlap clock")?;
    let start = reference["range"]["start"]
        .as_f64()
        .ok_or("missing overlap start")?
        / ticks;
    let duration = (reference["range"]["end"]
        .as_f64()
        .ok_or("missing overlap end")?
        - reference["range"]["start"]
            .as_f64()
            .ok_or("missing overlap start")?)
        / ticks;
    if !(0.0..=3.0).contains(&duration) || duration == 0.0 {
        return Err("overlap excerpt exceeds reviewed bound".into());
    }
    Ok((
        "owner-recorded-overlap-7".into(),
        excerpt(&root.join("261005-channel-1-10min.wav"), start, duration)?,
        json!({"reference_id":7,"project_sha256":OVERLAP_PROJECT_SHA256,"source_file":"261005-channel-1-10min.wav","start_seconds":start,"duration_seconds":duration,"active_speakers":reference["active_speakers"],"source_binding":reference["sources"],"verified_sources":verified,"limits":"recorded natural overlap with saved human active-speaker mask; development reference, not an independent per-sample speech audit"}),
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if ![3, 4].contains(&args.len()) {
        return Err(
            "usage: feasibility AUTHORIZED_TEST_AUDIO NEW_REPORT_JSON [PUBLIC_FIXTURE_DIRECTORY]"
                .into(),
        );
    }
    let root = Path::new(&args[1]);
    let project_bytes = std::fs::read(root.join("261005-supervised.cleanslate"))?;
    let project_sha256 = format!("{:x}", Sha256::digest(&project_bytes));
    if project_sha256 != REFERENCE_PROJECT_SHA256 {
        return Err("reviewed reference project identity mismatch; review bindings before changing this fixture".into());
    }
    let project: serde_json::Value = serde_json::from_slice(&project_bytes)?;
    // Check every additional PCM/source identity before any model is prepared.
    let mut category_cases = if args.len() == 4 {
        let mut cases = public_categories(Path::new(&args[3]))?;
        cases.push(recorded_overlap(root)?);
        cases
    } else {
        Vec::new()
    };
    // Verify the selected local files against their human-reference source binding.
    let mut source_hashes = Vec::new();
    for source in project["sources"].as_array().ok_or("missing sources")? {
        let track = source["track_id"].as_u64().ok_or("missing track")?;
        let path = root.join(format!("261005-channel-{track}.wav"));
        let hash = file_sha256(&path)?;
        if source["fingerprint"].as_str() != Some(&hash) {
            return Err(format!("source binding mismatch: {}", path.display()).into());
        }
        source_hashes.push(json!({"track":track,"sha256":hash}));
    }
    let mut cases: Vec<Case> = Vec::new();
    cases.push((
        "synthetic-silence".into(),
        vec![0.0; 48_000],
        json!({"truth":"constructed silence"}),
    ));
    let mut random = 1_u32;
    let noise = (0..48_000)
        .map(|_| {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            (random as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32 * 0.01
        })
        .collect();
    cases.push((
        "synthetic-white-noise".into(),
        noise,
        json!({"truth":"constructed noise"}),
    ));
    for reference in project["references"]
        .as_array()
        .ok_or("missing references")?
    {
        let id = reference["id"].as_u64().ok_or("missing id")?;
        let speakers = reference["active_speakers"]
            .as_array()
            .ok_or("missing speakers")?;
        let track = speakers.first().and_then(|s| s.as_u64()).unwrap_or(1);
        let ticks = project["ticks_per_second"]
            .as_f64()
            .ok_or("missing ticks")?;
        let start = reference["range"]["start"]
            .as_f64()
            .ok_or("missing start")?
            / ticks;
        let end = reference["range"]["end"].as_f64().ok_or("missing end")? / ticks;
        let path = root.join(format!("261005-channel-{track}.wav"));
        let data = excerpt(&path, start, (end - start).min(3.0))?;
        cases.push((format!("owner-reference-{id}"), data, json!({"reference_id":id,"source_file":path.file_name().unwrap().to_string_lossy(),"start_seconds":start,"active_speakers":speakers,"source_binding":reference["sources"],"limits":"development reference; not independent; speaker absence on another mic is not speech absence"})));
    }
    let speech = cases
        .iter()
        .find(|c| c.0 == "owner-reference-9")
        .ok_or("reference9 missing")?
        .1
        .clone();
    cases.push(("derived-quiet-speech".into(), speech.iter().map(|x| x*0.05).collect(), json!({"derived_from":9,"gain":0.05,"limits":"scaled speech, not human-labeled naturally quiet speech"})));
    cases.push((
        "derived-short-speech".into(),
        speech
            .iter()
            .take(7200)
            .copied()
            .chain(std::iter::repeat_n(0.0, 40_800))
            .collect(),
        json!({"derived_from":9,"limits":"150ms crop, not verified interjection"}),
    ));
    let other = &cases
        .iter()
        .find(|c| c.0 == "owner-reference-12")
        .ok_or("reference12 missing")?
        .1;
    let overlap = speech
        .iter()
        .zip(other.iter().cycle())
        .map(|(a, b)| (a + b) * 0.5)
        .collect();
    cases.push((
        "derived-speech-overlap".into(),
        overlap,
        json!({"derived_from":[9,12],"limits":"artificial mix; not natural human-labeled overlap"}),
    ));
    cases.append(&mut category_cases);
    let mut reports = Vec::new();
    for rate in [44_100, 48_000, 96_000] {
        let preparation = Instant::now();
        let mut detector = Detector::new(rate)?;
        let preparation_us = preparation.elapsed().as_micros();
        for (name, source, provenance) in &cases {
            // Fixture rate conversion only; linear interpolation is not the detector's FIR.
            let samples: Vec<_> = (0..source.len() * rate as usize / 48_000)
                .map(|i| {
                    let pos = i as f64 * 48_000.0 / f64::from(rate);
                    let index = pos.floor() as usize;
                    let a = source[index];
                    let b = source.get(index + 1).copied().unwrap_or(a);
                    a + (b - a) * (pos.fract() as f32)
                })
                .collect();
            let mut baseline = None;
            for block in [1, 17, 64, 257, 1024] {
                detector.reset();
                let mut events = Vec::with_capacity(samples.len() / rate as usize * 32 + 64);
                let mut timings = Vec::with_capacity(samples.len().div_ceil(block));
                let mut inference_blocks =
                    Vec::with_capacity(samples.len() / rate as usize * 32 + 64);
                let allocations_before = ALLOCATIONS.load(Ordering::Relaxed);
                for chunk in samples.chunks(block) {
                    let before = events.len();
                    let t = Instant::now();
                    detector.push(chunk, |e| events.push(e))?;
                    let elapsed = t.elapsed().as_nanos() as u64;
                    timings.push(elapsed);
                    if events.len() != before {
                        inference_blocks.push(elapsed);
                    }
                }
                let rust_allocations = ALLOCATIONS.load(Ordering::Relaxed) - allocations_before;
                let probabilities: Vec<_> = events.iter().map(|e| e.probability).collect();
                if let Some(ref expected) = baseline {
                    assert_eq!(expected, &probabilities, "block partition changed evidence");
                } else {
                    baseline = Some(probabilities.clone());
                }
                timings.sort_unstable();
                inference_blocks.sort_unstable();
                let bytes: Vec<_> = source.iter().flat_map(|s| s.to_le_bytes()).collect();
                reports.push(json!({"case":name,"provenance":provenance,"excerpt_f32_sha256":format!("{:x}",Sha256::digest(bytes)),"native_rate":rate,"block":block,"preparation_us":preparation_us,"frame_hop_ms":32,"fir_delay_native_samples":64,"fir_delay_ms":64_000.0/f64::from(rate),"events":probabilities,"model_calls":events.len(),"rust_allocations_during_push":rust_allocations,"allocation_limits":"excludes ORT native allocations; event/timing vectors preallocated","last_available_native":events.last().map(|e|e.available_at),"incomplete_tail_native_samples":samples.len() as u64-events.last().map(|e|e.available_at).unwrap_or(0),"push_ns_p50":timings[timings.len()/2],"push_ns_p95":timings[timings.len()*95/100],"push_ns_max":timings.last(),"inference_bearing_push_ns_p95":inference_blocks.get(inference_blocks.len()*95/100),"inference_bearing_push_ns_max":inference_blocks.last(),"total_push_ns":timings.iter().sum::<u64>()}));
            }
        }
    }
    let report = json!({"schema":1,"recipe":RECIPE,"model_sha256":MODEL_SHA256,"project_sha256":project_sha256,"verified_sources":source_hashes,"category_fixtures_enabled":args.len()==4,"limits":"synchronous allocating offline feasibility; ORT CPU1 thread; category smoke only, not per-frame quality, callback suitability or owner listening; no EOF padding/inference","runs":reports});
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    serde_json::to_writer_pretty(file, &report)?;
    println!(
        "PASS: {} rate/block/case runs, exact partition/reset evidence; report {}",
        reports.len(),
        args[2]
    );
    Ok(())
}
