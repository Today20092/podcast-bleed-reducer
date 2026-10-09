use crate::{RenderBlocker, Rounding, SourceTimebase, TimePosition, TimeRange};
mod accepted;
pub use accepted::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    Preserve,
    Bleed,
    Ignore,
}
#[derive(Clone, Debug)]
pub struct Label {
    pub track: usize,
    pub start: TimePosition,
    pub end: TimePosition,
    pub kind: LabelKind,
}
#[derive(Debug, PartialEq)]
pub struct Evidence {
    pub preservation_samples: usize,
    pub changed_preservation_samples: usize,
    /// Overlap counts use the first source's native grid; each preserved-track
    /// sample is counted separately above, including mixed-rate sources.
    pub overlap_samples: usize,
    pub changed_overlap_samples: usize,
    pub bleed_input_rms: f64,
    pub bleed_output_rms: f64,
}

pub fn evaluate(
    inputs: &[Vec<f32>],
    outputs: &[Vec<f32>],
    labels: &[Label],
    grids: &[SourceTimebase],
) -> Result<Evidence, RenderBlocker> {
    if inputs.is_empty() || inputs.len() != outputs.len() || inputs.len() != grids.len() {
        return Err(RenderBlocker(
            "evaluation sources and grids do not match".into(),
        ));
    }
    for ((input, output), grid) in inputs.iter().zip(outputs).zip(grids) {
        grid.duration()?;
        if input.len() != output.len()
            || input.len() as u64 != grid.frame_count
            || input.iter().chain(output).any(|s| !s.is_finite())
        {
            return Err(RenderBlocker(
                "evaluation audio is invalid or differs from its source grid".into(),
            ));
        }
    }
    let mut preservation_samples = 0;
    let mut changed_preservation_samples = 0;
    let mut bleed_samples = 0_u64;
    let (mut bleed_input_square, mut bleed_output_square) = (0.0_f64, 0.0_f64);
    let mut changed_ranges = Vec::<TimeRange>::new();
    for label in labels {
        let grid = *grids
            .get(label.track)
            .ok_or_else(|| RenderBlocker("evaluation label has unknown track".into()))?;
        let range = TimeRange::new(label.start, label.end)?.native_frames(grid)?;
        let mut run = None;
        for sample in range {
            match label.kind {
                LabelKind::Preserve => {
                    preservation_samples += 1;
                    if inputs[label.track][sample] != outputs[label.track][sample] {
                        changed_preservation_samples += 1;
                        run.get_or_insert(sample);
                    } else if let Some(start) = run.take() {
                        changed_ranges.push(TimeRange {
                            start: grid.time_at_frame(start as u64, Rounding::Floor)?,
                            end: grid.time_at_frame(sample as u64, Rounding::Ceil)?,
                        });
                    }
                }
                LabelKind::Bleed => {
                    bleed_samples += 1;
                    bleed_input_square += f64::from(inputs[label.track][sample]).powi(2);
                    bleed_output_square += f64::from(outputs[label.track][sample]).powi(2);
                }
                LabelKind::Ignore => {}
            }
        }
        if let Some(start) = run {
            changed_ranges.push(TimeRange {
                start: grid.time_at_frame(start as u64, Rounding::Floor)?,
                end: label.end,
            });
        }
    }
    changed_ranges.sort_unstable_by_key(|r| r.start);
    let mut merged = Vec::<TimeRange>::new();
    for range in changed_ranges {
        if let Some(previous) = merged.last_mut()
            && previous.end >= range.start
        {
            previous.end = previous.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    let preserved: Vec<_> = labels
        .iter()
        .filter(|l| l.kind == LabelKind::Preserve)
        .collect();
    let mut edges: Vec<_> = preserved.iter().flat_map(|l| [l.start, l.end]).collect();
    edges.sort_unstable();
    edges.dedup();
    let mut overlap_samples = 0;
    let mut changed_overlap_samples = 0;
    let reference = grids[0];
    for span in edges.windows(2) {
        let tracks: std::collections::HashSet<_> = preserved
            .iter()
            .filter(|l| l.start <= span[0] && l.end >= span[1])
            .map(|l| l.track)
            .collect();
        if tracks.len() < 2 {
            continue;
        }
        let end = span[1].min(reference.duration()?);
        let start = span[0].min(end);
        if start == end {
            continue;
        }
        let frames = TimeRange { start, end }.native_frames(reference)?;
        for sample in frames {
            overlap_samples += 1;
            let sample_start = reference.time_at_frame(sample as u64, Rounding::Floor)?;
            let sample_end = reference.time_at_frame(sample as u64 + 1, Rounding::Ceil)?;
            let first = merged.partition_point(|r| r.end <= sample_start);
            if merged.get(first).is_some_and(|r| r.start < sample_end) {
                changed_overlap_samples += 1;
            }
        }
    }
    let rms = |sum: f64| {
        if bleed_samples == 0 {
            0.0
        } else {
            (sum / bleed_samples as f64).sqrt()
        }
    };
    Ok(Evidence {
        preservation_samples,
        changed_preservation_samples,
        overlap_samples,
        changed_overlap_samples,
        bleed_input_rms: rms(bleed_input_square),
        bleed_output_rms: rms(bleed_output_square),
    })
}
