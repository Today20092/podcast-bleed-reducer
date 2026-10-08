use bwavfile::WaveReader;
use rust_app::{
    Rounding, RunRequest, SourceTimebase,
    release_gate::{Label, LabelKind, evaluate},
    run, time_from_seconds,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

struct Inspection {
    format: (u16, u16, u32, u16),
    time_reference: Option<u64>,
    samples: Vec<f32>,
}

fn inspect(path: &Path) -> Result<Inspection, Box<dyn std::error::Error>> {
    let mut reader = WaveReader::open(path)?;
    let count = usize::try_from(reader.frame_length()?)?;
    let format = reader.format()?;
    let time_reference = reader
        .broadcast_extension()?
        .map(|bext| bext.time_reference);
    let mut samples = format.create_frame_buffer::<f32>(count);
    let read = reader.audio_frame_reader()?.read_frames(&mut samples)?;
    samples.truncate(usize::try_from(read)?);
    Ok(Inspection {
        format: (
            format.tag,
            format.channel_count,
            format.sample_rate,
            format.bits_per_sample,
        ),
        time_reference,
        samples,
    })
}

fn integrity(input: &Inspection, output: &Inspection) -> bool {
    input.format == output.format
        && input.time_reference == output.time_reference
        && input.samples.len() == output.samples.len()
        && output.samples.iter().all(|sample| sample.is_finite())
        && output
            .samples
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
            <= input
                .samples
                .iter()
                .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

fn read_labels(path: &Path, stems: &[PathBuf]) -> Result<Vec<Label>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for line in fs::read_to_string(path)?.lines().skip(1) {
        let fields: Vec<_> = line
            .split(',')
            .map(|field| field.trim_matches('"'))
            .collect();
        if fields.len() < 4 {
            continue;
        }
        let Some(track) = stems
            .iter()
            .position(|stem| stem.file_name().and_then(|n| n.to_str()) == Some(fields[0]))
        else {
            continue;
        };
        let kind = match fields[3] {
            "direct-speech" | "laughter-breath" | "uncertain" => LabelKind::Preserve,
            "bleed" => LabelKind::Bleed,
            _ => LabelKind::Ignore,
        };
        result.push(Label {
            track,
            start: time_from_seconds(fields[1].parse::<f64>()?, Rounding::Floor)?,
            end: time_from_seconds(fields[2].parse::<f64>()?, Rounding::Ceil)?,
            kind,
        });
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.first().is_some_and(|arg| arg == "--cancel") {
        if args.len() < 4 {
            return Err("usage: release_gate --cancel <output-dir> <stem.wav>...".into());
        }
        let output_dir = args[1].clone();
        return match run(RunRequest {
            stems: args[2..].to_vec(),
            output_dir: output_dir.clone(),
            regions: vec![],
            protections: vec![],
            cancelled: Some(Arc::new(AtomicBool::new(true))),
        }) {
            Err(error) if error.to_string() == "render cancelled" && !output_dir.exists() => {
                println!("safe cancellation: pass");
                Ok(())
            }
            Err(error) => Err(error.into()),
            Ok(_) => Err("safe cancellation check unexpectedly rendered output".into()),
        };
    }
    if args.len() < 4 {
        return Err("usage: release_gate <labels.csv|-> <output-dir> <stem.wav>...".into());
    }
    let label_path = &args[0];
    let output_dir = args[1].clone();
    let stems = args[2..].to_vec();
    let input_inspections = if label_path.as_os_str() == "-" {
        vec![]
    } else {
        stems
            .iter()
            .map(|path| inspect(path))
            .collect::<Result<_, _>>()?
    };
    let started = Instant::now();
    let result = run(RunRequest {
        stems: stems.clone(),
        output_dir: output_dir.clone(),
        regions: vec![],
        protections: vec![],
        cancelled: Some(Arc::new(AtomicBool::new(false))),
    })?;
    let output_inspections = if input_inspections.is_empty() {
        vec![]
    } else {
        result
            .stems
            .iter()
            .map(|path| inspect(path))
            .collect::<Result<_, _>>()?
    };
    let integrity_passed = if input_inspections.is_empty() {
        stems
            .iter()
            .zip(&result.stems)
            .try_fold(true, |passed, (input, output)| {
                Ok::<_, Box<dyn std::error::Error>>(
                    passed && integrity(&inspect(input)?, &inspect(output)?),
                )
            })?
    } else {
        input_inspections
            .iter()
            .zip(&output_inspections)
            .all(|(input, output)| integrity(input, output))
    };
    let sample_count = if let Some(input) = input_inspections.first() {
        input.samples.len()
    } else {
        usize::try_from(WaveReader::open(&result.stems[0])?.frame_length()?)?
    };
    let mut evidence = format!(
        "# CleanSlate release-gate evidence\n\n- elapsed_seconds: {:.3}\n- stems: {}\n- sample_count: {}\n- atomic_render: pass\n- independently_verified_integrity: {}\n",
        started.elapsed().as_secs_f64(),
        result.stems.len(),
        sample_count,
        if integrity_passed { "pass" } else { "fail" }
    );
    if !input_inspections.is_empty() {
        let grids = input_inspections
            .iter()
            .map(|input| SourceTimebase::new(input.format.2, input.samples.len() as u64))
            .collect::<Result<Vec<_>, _>>()?;
        let input_samples = input_inspections
            .into_iter()
            .map(|stem| stem.samples)
            .collect::<Vec<_>>();
        let output_samples = output_inspections
            .into_iter()
            .map(|stem| stem.samples)
            .collect::<Vec<_>>();
        let measured = evaluate(
            &input_samples,
            &output_samples,
            &read_labels(label_path, &stems)?,
            &grids,
        )?;
        evidence.push_str(&format!("- preservation_samples: {}\n- changed_preservation_samples: {}\n- overlap_samples: {}\n- changed_overlap_samples: {}\n- bleed_input_rms: {:.9}\n- bleed_output_rms: {:.9}\n", measured.preservation_samples, measured.changed_preservation_samples, measured.overlap_samples, measured.changed_overlap_samples, measured.bleed_input_rms, measured.bleed_output_rms));
        let passed = integrity_passed
            && measured.preservation_samples != 0
            && measured.changed_preservation_samples == 0
            && measured.overlap_samples != 0
            && measured.changed_overlap_samples == 0
            && measured.bleed_output_rms < measured.bleed_input_rms;
        evidence.push_str(&format!(
            "- automated_gate: {}\n",
            if passed { "pass" } else { "fail" }
        ));
    }
    if !integrity_passed {
        evidence.push_str("- automated_gate: fail\n");
    }
    evidence.push_str("\n## Human listening sign-off\n\n- [ ] Deterministic transitions are artifact-free.\n- [ ] Labeled reference transitions are artifact-free.\n- [ ] Full-session gross spot-check passed.\n");
    fs::write(output_dir.join("release-gate.md"), evidence)?;
    println!("{}", output_dir.join("release-gate.md").display());
    if fs::read_to_string(output_dir.join("release-gate.md"))?.contains("automated_gate: fail") {
        return Err("release acceptance gate failed".into());
    }
    Ok(())
}
