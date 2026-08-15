#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    Preserve,
    Bleed,
    Ignore,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub track: usize,
    pub start: usize,
    pub end: usize,
    pub kind: LabelKind,
}

#[derive(Debug, PartialEq)]
pub struct Evidence {
    pub preservation_samples: usize,
    pub changed_preservation_samples: usize,
    pub overlap_samples: usize,
    pub changed_overlap_samples: usize,
    pub bleed_input_rms: f64,
    pub bleed_output_rms: f64,
}

pub fn evaluate(inputs: &[Vec<f32>], outputs: &[Vec<f32>], labels: &[Label]) -> Evidence {
    let mut preservation_samples = 0;
    let mut changed_preservation_samples = 0;
    let mut bleed_samples = 0_u64;
    let (mut bleed_input_square, mut bleed_output_square) = (0.0_f64, 0.0_f64);
    let sample_count = inputs.first().map_or(0, Vec::len);
    let mut first_preserved_track = vec![usize::MAX; sample_count];
    let mut overlap = vec![false; sample_count];
    let mut changed_preservation = vec![false; sample_count];

    for label in labels {
        let end = label.end.min(sample_count);
        for sample in label.start.min(end)..end {
            match label.kind {
                LabelKind::Preserve => {
                    preservation_samples += 1;
                    if first_preserved_track[sample] == usize::MAX {
                        first_preserved_track[sample] = label.track;
                    } else if first_preserved_track[sample] != label.track {
                        overlap[sample] = true;
                    }
                    if inputs[label.track][sample] != outputs[label.track][sample] {
                        changed_preservation_samples += 1;
                        changed_preservation[sample] = true;
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
    }

    let overlap_samples = overlap.iter().filter(|overlap| **overlap).count();
    let changed_overlap_samples = overlap
        .iter()
        .enumerate()
        .filter(|(sample, overlap)| **overlap && changed_preservation[*sample])
        .count();
    let rms = |sum: f64| {
        if bleed_samples == 0 {
            0.0
        } else {
            (sum / bleed_samples as f64).sqrt()
        }
    };
    Evidence {
        preservation_samples,
        changed_preservation_samples,
        overlap_samples,
        changed_overlap_samples,
        bleed_input_rms: rms(bleed_input_square),
        bleed_output_rms: rms(bleed_output_square),
    }
}
