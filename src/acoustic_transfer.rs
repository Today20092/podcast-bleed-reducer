//! Bounded, reference-only development qualification of a regularized two-tap transfer.
use crate::{PreparedAnalysisAudio, Project, RenderBlocker, TICKS_PER_SECOND, TrackId};
use nalgebra::{Matrix2, Vector2};
use std::sync::atomic::AtomicBool;
const GUARD: usize = 196;
const MAX_LAG: isize = 192; // 12 ms on the analysis grid; never a source alignment change.
const RIDGE: f64 = 1e-4;
#[derive(Clone, Debug, serde::Serialize)]
pub struct AcousticEstimatorComparison {
    pub source: TrackId,
    pub target: TrackId,
    pub scalar_explained: f32,
    pub fir_explained: f32,
    pub worst_reference_explained: f32,
    pub residual_improvement_db: f32,
    pub fir_selected: bool,
    pub validation_samples: u64,
    pub lag_samples: isize,
}
#[derive(Clone, Debug)]
pub(crate) struct TransferCandidate {
    pub comparison: AcousticEstimatorComparison,
    pub lag: isize,
    pub coefficients: [f64; 2],
    pub validation_residual_db: f32,
}
struct Samples {
    x: Vec<f32>,
    y: Vec<f32>,
}
fn visit(
    project: &Project,
    audio: &[PreparedAnalysisAudio],
    source: usize,
    target: usize,
    validation: bool,
    cancelled: Option<&AtomicBool>,
    mut use_samples: impl FnMut(Samples),
) -> Result<(), RenderBlocker> {
    // Union identical truth before splitting: no physical sample may train one
    // reference and validate another overlapping reference.
    let mut ranges: Vec<_> = project
        .reference_labels()
        .iter()
        .filter(|label| label.active_speakers == [project.sources[source].speaker_id])
        .map(|label| (label.range.start, label.range.end))
        .collect();
    ranges.sort_unstable();
    let mut united: Vec<(u64, u64)> = Vec::new();
    for (start, end) in ranges {
        if let Some(previous) = united.last_mut().filter(|previous| start <= previous.1) {
            previous.1 = previous.1.max(end);
        } else {
            united.push((start, end));
        }
    }
    for (start, end) in united {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let start = start.div_ceil(TICKS_PER_SECOND / 16_000) as usize + GUARD;
        let end = (end / (TICKS_PER_SECOND / 16_000)) as usize;
        let end = end.saturating_sub(GUARD);
        if end <= start + 96 {
            continue;
        }
        let count = ((end - start) / 3).min(2048);
        let left = if validation { end - count } else { start };
        use_samples(Samples {
            x: audio[source].read_frames(left - GUARD..left + count + GUARD)?,
            y: audio[target].read_frames(left..left + count)?,
        });
    }
    Ok(())
}
fn x(samples: &Samples, row: usize, lag: isize) -> f64 {
    f64::from(samples.x[(GUARD as isize + row as isize - lag) as usize])
}
pub(crate) fn qualify(
    project: &Project,
    audio: &[PreparedAnalysisAudio],
    source: usize,
    target: usize,
    cancelled: Option<&AtomicBool>,
) -> Result<Option<TransferCandidate>, RenderBlocker> {
    let mut sums = vec![[0.0_f64; 3]; (MAX_LAG * 2 + 1) as usize];
    visit(
        project,
        audio,
        source,
        target,
        false,
        cancelled,
        |samples| {
            for lag in -MAX_LAG..=MAX_LAG {
                let total = &mut sums[(lag + MAX_LAG) as usize];
                for (row, &y) in samples.y.iter().enumerate() {
                    let (x, y) = (x(&samples, row, lag), f64::from(y));
                    total[0] += x * x;
                    total[1] += y * y;
                    total[2] += x * y;
                }
            }
        },
    )?;
    let Some((best, _)) = sums
        .iter()
        .enumerate()
        .filter(|(_, sum)| sum[0] > 1e-24 && sum[1] > 1e-24)
        .max_by(|(_, a), (_, b)| {
            ((a[2] * a[2]) / (a[0] * a[1])).total_cmp(&((b[2] * b[2]) / (b[0] * b[1])))
        })
    else {
        return Ok(None);
    };
    let scalar_lag = best as isize - MAX_LAG;
    let scalar_gain = sums[best][2] / sums[best][0];
    let mut best_fir: Option<(isize, [f64; 2], f64)> = None;
    for lag in [scalar_lag - 1, scalar_lag] {
        if lag < -MAX_LAG || lag + 1 > MAX_LAG {
            continue;
        }
        let mut gram = Matrix2::<f64>::zeros();
        let mut cross = Vector2::<f64>::zeros();
        let mut yy = 0.0;
        let mut n = 0_u64;
        visit(
            project,
            audio,
            source,
            target,
            false,
            cancelled,
            |samples| {
                for (row, &y) in samples.y.iter().enumerate() {
                    let v = Vector2::new(x(&samples, row, lag), x(&samples, row, lag + 1));
                    gram += v * v.transpose();
                    cross += v * f64::from(y);
                    yy += f64::from(y).powi(2);
                    n += 1;
                }
            },
        )?;
        if n == 0 || yy <= 1e-24 || gram.trace() <= 1e-24 {
            continue;
        }
        let regularization = gram.trace() / 2.0 * RIDGE;
        let Some(solver) = (gram + Matrix2::identity() * regularization).cholesky() else {
            continue;
        };
        let solved = solver.solve(&cross);
        if solved.iter().any(|value| !value.is_finite()) {
            continue;
        }
        let error = (yy - 2.0 * solved.dot(&cross) + solved.dot(&(gram * solved))).max(0.0) / yy;
        if best_fir.as_ref().is_none_or(|(_, _, old)| error < *old) {
            best_fir = Some((lag, [solved[0], solved[1]], error));
        }
    }
    let Some((lag, coefficients, _)) = best_fir else {
        return Ok(None);
    };
    let (mut scalar_error, mut fir_error, mut power, mut count, mut worst) =
        (0.0_f64, 0.0_f64, 0.0_f64, 0_u64, 1.0_f64);
    visit(project, audio, source, target, true, cancelled, |samples| {
        let (mut local_error, mut local_power) = (0.0_f64, 0.0_f64);
        for (row, &y) in samples.y.iter().enumerate() {
            let y = f64::from(y);
            let predicted = coefficients[0] * x(&samples, row, lag)
                + coefficients[1] * x(&samples, row, lag + 1);
            scalar_error += (y - scalar_gain * x(&samples, row, scalar_lag)).powi(2);
            fir_error += (y - predicted).powi(2);
            power += y * y;
            count += 1;
            local_error += (y - predicted).powi(2);
            local_power += y * y;
        }
        worst = worst.min((1.0 - local_error / local_power.max(1e-24)).clamp(0.0, 1.0));
    })?;
    if count < 96 || power <= 1e-24 {
        return Ok(None);
    }
    let scalar = (1.0 - scalar_error / power).clamp(0.0, 1.0);
    let fir = (1.0 - fir_error / power).clamp(0.0, 1.0);
    let improvement =
        10.0 * (scalar_error.max(power * 1e-12) / fir_error.max(power * 1e-12)).log10();
    let selected = fir >= 0.85 && worst >= 0.7 && improvement >= 3.0;
    Ok(Some(TransferCandidate {
        comparison: AcousticEstimatorComparison {
            source: project.sources[source].track_id,
            target: project.sources[target].track_id,
            scalar_explained: scalar as f32,
            fir_explained: fir as f32,
            worst_reference_explained: worst as f32,
            residual_improvement_db: improvement.clamp(-120.0, 120.0) as f32,
            fir_selected: selected,
            validation_samples: count,
            lag_samples: lag,
        },
        lag,
        coefficients,
        validation_residual_db: (10.0 * (fir_error / power).max(1e-12).log10()) as f32,
    }))
}
pub(crate) fn residual_frames(
    audio: &[PreparedAnalysisAudio],
    models: &[(usize, usize, TransferCandidate)],
    cancelled: Option<&AtomicBool>,
) -> Result<Vec<Vec<f32>>, RenderBlocker> {
    if models.is_empty() {
        return Ok(vec![]);
    }
    let length = audio
        .iter()
        .map(|copy| copy.identity.frame_count as usize)
        .max()
        .unwrap_or(0);
    let mut results = vec![Vec::with_capacity(length.div_ceil(512)); models.len()];
    for start in (0..length).step_by(4096) {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let end = (start + 4096).min(length);
        let left = start.saturating_sub(GUARD);
        let right = (end + GUARD).min(length);
        let blocks = audio
            .iter()
            .map(|copy| {
                let available = copy.identity.frame_count as usize;
                let mut values = copy.read_frames(left.min(available)..right.min(available))?;
                values.resize(right - left, 0.0);
                Ok(values)
            })
            .collect::<Result<Vec<_>, RenderBlocker>>()?;
        for (model, (source, target, candidate)) in models.iter().enumerate() {
            for frame_start in (start..end).step_by(512) {
                let frame_end = (frame_start + 512).min(end);
                let (mut error, mut power) = (0.0_f64, 0.0_f64);
                for t in frame_start..frame_end {
                    let sample = |lag: isize| {
                        let at = t as isize - lag;
                        if at < 0 || at >= length as isize {
                            0.0
                        } else {
                            f64::from(blocks[*source][at as usize - left])
                        }
                    };
                    let predicted = candidate.coefficients[0] * sample(candidate.lag)
                        + candidate.coefficients[1] * sample(candidate.lag + 1);
                    let target = f64::from(blocks[*target][t - left]);
                    error += (target - predicted).powi(2);
                    power += target * target;
                }
                let excess = 10.0 * (error / power.max(1e-24)).max(1e-12).log10()
                    - f64::from(candidate.validation_residual_db);
                results[model].push(excess.clamp(-120.0, 120.0) as f32);
            }
        }
    }
    Ok(results)
}
