//! Read-only source evaluation; writes metrics, never audio.
use rust_app::{CalibrationExample, CalibrationKind, OwnershipSettings, SourceStamp};
use std::{fs, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let usage = "ownership_evaluate <labels.csv> <excerpt|full|settings.json> <development|holdout|all|settings> <new-output.json> <stem.wav> ...";
    let labels = args.next().ok_or(usage)?;
    let settings = args.next().ok_or(usage)?;
    let split = args.next().ok_or(usage)?;
    let output = args.next().ok_or(usage)?;
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    if paths.is_empty() {
        return Err(usage.into());
    }
    let guided = if settings == "excerpt" || settings == "full" {
        // Verified exact PCM matches in all three stems at five development windows.
        // BEXT is one sample earlier and is not the PCM alignment authority here.
        let offset = if settings == "full" { 8_973_765 } else { 0 };
        let mut examples = Vec::new();
        for (file, kind, start, end) in [
            (
                "00005603_A01.wav",
                CalibrationKind::SoloSpeech,
                7.386,
                7.770,
            ),
            (
                "00005603_A03.wav",
                CalibrationKind::SoloSpeech,
                5.802,
                6.186,
            ),
            (
                "00005603_A02.wav",
                CalibrationKind::SoloSpeech,
                66.099,
                66.611,
            ),
            ("00005603_A03.wav", CalibrationKind::RoomTone, 3.345, 3.878),
        ] {
            let file = if settings == "full" {
                file.replace("00005603", "chopshow podcast 155")
            } else {
                file.into()
            };
            let track = paths
                .iter()
                .position(|p| p.file_name().and_then(|n| n.to_str()) == Some(file.as_str()))
                .ok_or("preset requires the original reference source filenames")?;
            examples.push(CalibrationExample {
                kind,
                track,
                start: offset + (start * 48_000.0_f64).round() as usize,
                end: offset + (end * 48_000.0_f64).round() as usize,
            });
        }
        OwnershipSettings {
            enabled: true,
            examples,
            sources: paths
                .iter()
                .map(|p| SourceStamp::read(p))
                .collect::<Result<_, _>>()?,
        }
    } else {
        serde_json::from_slice(&fs::read(settings)?)?
    };
    if !guided.enabled || guided.examples.is_empty() {
        return Err("guided comparison requires enabled settings with calibration examples".into());
    }
    let report = if split == "settings" {
        serde_json::to_value(&guided)?
    } else {
        rust_app::detector_quality::compare_ownership(
            &paths,
            &fs::read_to_string(labels)?,
            &guided,
            split.to_str().ok_or("invalid split")?,
        )?
    };
    // Never overwrite a source, settings file, or an earlier measurement.
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?
        .write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    Ok(())
}
