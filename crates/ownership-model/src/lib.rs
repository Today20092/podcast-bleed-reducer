//! Maintained binary training plus explicit, portable inference metadata.
use linfa::{Dataset, traits::Fit};
use ndarray::{Array1, Array2};
use std::sync::atomic::AtomicBool;

pub const OWNERSHIP_MODEL_RECIPE: &str = "linfa081-binary-train-zscore-mask-l2mean0.01-v1";

#[derive(Debug, Clone)]
pub struct RenderBlocker(pub String);

impl std::fmt::Display for RenderBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RenderBlocker {}

fn check_cancelled(cancelled: Option<&AtomicBool>) -> Result<(), RenderBlocker> {
    if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
        Err(RenderBlocker("preview cancelled".into()))
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct OwnershipTrainingExample {
    pub features: Vec<f64>,
    pub owns_speech: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OwnershipTrainingSettings {
    pub mean_loss_l2: f64,
    pub max_iterations: u64,
}
impl Default for OwnershipTrainingSettings {
    fn default() -> Self {
        Self {
            mean_loss_l2: 0.01,
            max_iterations: 300,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OwnershipFitDiagnostics {
    pub rows: usize,
    pub positive_rows: usize,
    pub penalized_mean_loss: f64,
    pub gradient_norm: f64,
    pub library_prediction_max_error: f64,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnershipModel {
    schema: u32,
    recipe: String,
    feature_names: Vec<String>,
    means: Vec<f64>,
    scales: Vec<f64>,
    retained: Vec<usize>,
    coefficients: Vec<f64>,
    intercept: f64,
    positive_label: bool,
    settings: OwnershipTrainingSettings,
    diagnostics: OwnershipFitDiagnostics,
}
impl OwnershipModel {
    /// Callers supply explicit human truth; production rows are built from Project labels.
    pub fn train(
        feature_names: Vec<String>,
        examples: &[OwnershipTrainingExample],
        settings: OwnershipTrainingSettings,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        check_cancelled(cancelled)?;
        let dimensions = feature_names.len();
        if dimensions == 0
            || dimensions > 64
            || examples.len() > 8192
            || examples.iter().any(|row| {
                row.features.len() != dimensions
                    || row.features.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
            })
        {
            return Err(RenderBlocker(
                "Ownership training needs bounded, finite, consistently shaped features.".into(),
            ));
        }
        let positive = examples.iter().filter(|row| row.owns_speech).count();
        if positive < 3 || examples.len().saturating_sub(positive) < 3 {
            return Err(RenderBlocker("Ownership training needs at least three usable human positive and negative frames.".into()));
        }
        if !settings.mean_loss_l2.is_finite()
            || settings.mean_loss_l2 <= 0.0
            || settings.mean_loss_l2 > 10.0
            || settings.max_iterations == 0
            || settings.max_iterations > 1000
        {
            return Err(RenderBlocker(
                "Invalid bounded ownership training configuration.".into(),
            ));
        }
        let mut ordered: Vec<_> = examples.iter().collect();
        ordered.sort_by(|a, b| {
            a.owns_speech.cmp(&b.owns_speech).then_with(|| {
                a.features
                    .iter()
                    .zip(&b.features)
                    .map(|(a, b)| a.total_cmp(b))
                    .find(|order| !order.is_eq())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        let means: Vec<_> = (0..dimensions)
            .map(|col| {
                ordered.iter().map(|row| row.features[col]).sum::<f64>() / ordered.len() as f64
            })
            .collect();
        let scales: Vec<_> = (0..dimensions)
            .map(|col| {
                (ordered
                    .iter()
                    .map(|row| (row.features[col] - means[col]).powi(2))
                    .sum::<f64>()
                    / ordered.len() as f64)
                    .sqrt()
            })
            .collect();
        let retained: Vec<_> = scales
            .iter()
            .enumerate()
            .filter(|(_, scale)| **scale > 1e-8)
            .map(|(col, _)| col)
            .collect();
        if retained.is_empty() {
            return Err(RenderBlocker(
                "Ownership reference features are constant; choose contrasting examples.".into(),
            ));
        }
        let records = Array2::from_shape_vec(
            (ordered.len(), retained.len()),
            ordered
                .iter()
                .flat_map(|row| {
                    retained
                        .iter()
                        .map(|&col| (row.features[col] - means[col]) / scales[col])
                })
                .collect(),
        )
        .map_err(|e| RenderBlocker(e.to_string()))?;
        let targets = Array1::from_iter(ordered.iter().map(|row| row.owns_speech));
        let fitted = linfa_logistic::LogisticRegression::<f64>::default()
            .alpha(settings.mean_loss_l2 * ordered.len() as f64)
            .max_iterations(settings.max_iterations)
            .gradient_tolerance(1e-7)
            .fit(&Dataset::new(records.clone(), targets))
            .map_err(|e| RenderBlocker(format!("Ownership training failed: {e}")))?;
        check_cancelled(cancelled)?;
        // The released trainer chooses positive class from counts/encounter order.
        // Normalize to application truth rather than trusting prose label ordering.
        let orientation = if fitted.labels().pos.class { 1.0 } else { -1.0 };
        let mut model = Self {
            schema: 1,
            recipe: OWNERSHIP_MODEL_RECIPE.into(),
            feature_names,
            means,
            scales,
            retained,
            coefficients: fitted
                .params()
                .iter()
                .map(|value| value * orientation)
                .collect(),
            intercept: fitted.intercept() * orientation,
            positive_label: true,
            settings,
            diagnostics: OwnershipFitDiagnostics {
                rows: ordered.len(),
                positive_rows: positive,
                penalized_mean_loss: 0.0,
                gradient_norm: 0.0,
                library_prediction_max_error: 0.0,
            },
        };
        let mut gradient = vec![0.0; model.coefficients.len() + 1];
        let mut loss = 0.0;
        let library = fitted.predict_probabilities(&records);
        for (i, row) in ordered.iter().enumerate() {
            let x = records.row(i);
            let z = model.intercept
                + x.iter()
                    .zip(&model.coefficients)
                    .map(|(x, w)| x * w)
                    .sum::<f64>();
            let estimate = sigmoid(z);
            model.diagnostics.library_prediction_max_error =
                model.diagnostics.library_prediction_max_error.max(
                    (estimate
                        - if orientation > 0.0 {
                            library[i]
                        } else {
                            1.0 - library[i]
                        })
                    .abs(),
                );
            let y = f64::from(row.owns_speech);
            loss += z.max(0.0) - y * z + (-z.abs()).exp().ln_1p();
            for (j, x) in x.iter().enumerate() {
                gradient[j] += (estimate - y) * x / ordered.len() as f64;
            }
            *gradient.last_mut().unwrap() += (estimate - y) / ordered.len() as f64;
        }
        loss /= ordered.len() as f64;
        for (j, w) in model.coefficients.iter().enumerate() {
            loss += settings.mean_loss_l2 * w * w / 2.0;
            gradient[j] += settings.mean_loss_l2 * w;
        }
        model.diagnostics.penalized_mean_loss = loss;
        model.diagnostics.gradient_norm = gradient.iter().map(|v| v * v).sum::<f64>().sqrt();
        model.validate()?;
        let prior = positive as f64 / ordered.len() as f64;
        let prior_loss = -prior * prior.ln() - (1.0 - prior) * (1.0 - prior).ln();
        if loss >= prior_loss - 1e-4 {
            return Err(RenderBlocker("Ownership references do not support a useful converged model; choose contrasting examples.".into()));
        }
        Ok(model)
    }
    fn validate(&self) -> Result<(), RenderBlocker> {
        let d = self.feature_names.len();
        if self.schema != 1
            || self.recipe != OWNERSHIP_MODEL_RECIPE
            || !self.positive_label
            || d == 0
            || d > 64
            || !self.settings.mean_loss_l2.is_finite()
            || self.settings.mean_loss_l2 <= 0.0
            || self.settings.mean_loss_l2 > 10.0
            || self.settings.max_iterations == 0
            || self.settings.max_iterations > 1000
            || self.diagnostics.rows > 8192
            || self.diagnostics.positive_rows < 3
            || self
                .diagnostics
                .rows
                .saturating_sub(self.diagnostics.positive_rows)
                < 3
            || self.means.len() != d
            || self.scales.len() != d
            || self.retained.is_empty()
            || self.retained
                != self
                    .scales
                    .iter()
                    .enumerate()
                    .filter(|(_, scale)| **scale > 1e-8)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>()
            || self.coefficients.len() != self.retained.len()
            || self.retained.windows(2).any(|w| w[0] >= w[1])
            || self
                .retained
                .iter()
                .any(|&i| i >= d || self.scales[i] <= 1e-8)
            || self
                .means
                .iter()
                .chain(&self.scales)
                .chain(&self.coefficients)
                .any(|v| !v.is_finite())
            || !self.intercept.is_finite()
            || !self.diagnostics.gradient_norm.is_finite()
            || self.diagnostics.gradient_norm > 1e-5
            || !self.diagnostics.penalized_mean_loss.is_finite()
            || self.diagnostics.penalized_mean_loss < 0.0
            || !self.diagnostics.library_prediction_max_error.is_finite()
            || self.scales.iter().any(|scale| *scale < 0.0)
            || self.diagnostics.library_prediction_max_error > 1e-12
        {
            return Err(RenderBlocker(format!(
                "Ownership model has invalid metadata or failed numerical convergence (gradient {}, prediction difference {}).",
                self.diagnostics.gradient_norm, self.diagnostics.library_prediction_max_error
            )));
        }
        Ok(())
    }
    pub fn estimate(&self, features: &[f64]) -> Result<f64, RenderBlocker> {
        self.validate()?;
        if features.len() != self.means.len()
            || features.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return Err(RenderBlocker(
                "Ownership inference features are invalid.".into(),
            ));
        }
        let z = self.intercept
            + self
                .retained
                .iter()
                .zip(&self.coefficients)
                .map(|(&i, w)| ((features[i] - self.means[i]) / self.scales[i]) * w)
                .sum::<f64>();
        if !z.is_finite() {
            return Err(RenderBlocker("Ownership predictor is not finite.".into()));
        }
        Ok(sigmoid(z))
    }
    pub fn diagnostics(&self) -> &OwnershipFitDiagnostics {
        &self.diagnostics
    }
    pub fn matches_recipe(&self, names: &[String], settings: OwnershipTrainingSettings) -> bool {
        self.feature_names == names && self.settings == settings
    }
    /// A restored cache's recorded diagnostics are not proof of its present coefficients.
    pub fn validate_truth_rows(
        &self,
        rows: &[OwnershipTrainingExample],
    ) -> Result<(), RenderBlocker> {
        self.validate()?;
        if rows.len() != self.diagnostics.rows
            || rows.iter().filter(|row| row.owns_speech).count() != self.diagnostics.positive_rows
            || rows.iter().any(|row| {
                row.features.len() != self.means.len()
                    || row.features.iter().any(|v| !v.is_finite())
            })
        {
            return Err(RenderBlocker(
                "Saved ownership model does not match current human training truth.".into(),
            ));
        }
        let mut recomputed_mask = vec![];
        for col in 0..self.means.len() {
            let mean = rows.iter().map(|row| row.features[col]).sum::<f64>() / rows.len() as f64;
            let scale = (rows
                .iter()
                .map(|row| (row.features[col] - mean).powi(2))
                .sum::<f64>()
                / rows.len() as f64)
                .sqrt();
            if scale > 1e-8 {
                recomputed_mask.push(col);
            }
            if (mean - self.means[col]).abs() > 1e-9 || (scale - self.scales[col]).abs() > 1e-9 {
                return Err(RenderBlocker(
                    "Saved ownership normalization differs from current training truth.".into(),
                ));
            }
        }
        if recomputed_mask != self.retained {
            return Err(RenderBlocker(
                "Saved ownership feature mask differs from current training truth.".into(),
            ));
        }
        let mut gradient = vec![0.0; self.coefficients.len() + 1];
        let mut loss = 0.0;
        for row in rows {
            let residual = self.estimate(&row.features)? - f64::from(row.owns_speech);
            let z = self.intercept
                + self
                    .retained
                    .iter()
                    .zip(&self.coefficients)
                    .map(|(&col, w)| (row.features[col] - self.means[col]) / self.scales[col] * w)
                    .sum::<f64>();
            loss += z.max(0.0) - f64::from(row.owns_speech) * z + (-z.abs()).exp().ln_1p();
            for (j, &col) in self.retained.iter().enumerate() {
                gradient[j] += residual * (row.features[col] - self.means[col])
                    / self.scales[col]
                    / rows.len() as f64;
            }
            *gradient.last_mut().unwrap() += residual / rows.len() as f64;
        }
        loss /= rows.len() as f64;
        for (j, w) in self.coefficients.iter().enumerate() {
            gradient[j] += self.settings.mean_loss_l2 * w;
            loss += self.settings.mean_loss_l2 * w * w / 2.0;
        }
        let prior = self.diagnostics.positive_rows as f64 / rows.len() as f64;
        let prior_loss = -prior * prior.ln() - (1.0 - prior) * (1.0 - prior).ln();
        if !loss.is_finite()
            || loss >= prior_loss - 1e-4
            || (loss - self.diagnostics.penalized_mean_loss).abs() > 1e-9
        {
            return Err(RenderBlocker(
                "Saved ownership model no longer has its useful training objective.".into(),
            ));
        }
        if gradient.iter().map(|v| v * v).sum::<f64>().sqrt() > 1e-5 {
            return Err(RenderBlocker(
                "Saved ownership coefficients fail convergence against current human truth.".into(),
            ));
        }
        Ok(())
    }
    pub fn to_json(&self) -> Result<String, RenderBlocker> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| RenderBlocker(e.to_string()))
    }
    pub fn from_json(json: &str) -> Result<Self, RenderBlocker> {
        let model: Self = serde_json::from_str(json).map_err(|e| RenderBlocker(e.to_string()))?;
        model.validate()?;
        Ok(model)
    }

    /// Validate once outside realtime processing, then use the allocation-free copy.
    pub fn predictor(&self) -> Result<OwnershipPredictor, RenderBlocker> {
        self.validate()?;
        let mut predictor = OwnershipPredictor {
            dimensions: self.means.len(),
            retained: self.retained.len(),
            indices: [0; 64],
            means: [0.0; 64],
            scales: [1.0; 64],
            weights: [0.0; 64],
            intercept: self.intercept,
        };
        for (slot, &index) in self.retained.iter().enumerate() {
            predictor.indices[slot] = index;
            predictor.means[slot] = self.means[index];
            predictor.scales[slot] = self.scales[index];
            predictor.weights[slot] = self.coefficients[slot];
        }
        Ok(predictor)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OwnershipPredictor {
    dimensions: usize,
    retained: usize,
    indices: [usize; 64],
    means: [f64; 64],
    scales: [f64; 64],
    weights: [f64; 64],
    intercept: f64,
}

impl OwnershipPredictor {
    pub fn estimate(&self, features: &[f64]) -> Option<f64> {
        if features.len() != self.dimensions
            || features.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return None;
        }
        let z = self.intercept
            + (0..self.retained)
                .map(|slot| {
                    ((features[self.indices[slot]] - self.means[slot]) / self.scales[slot])
                        * self.weights[slot]
                })
                .sum::<f64>();
        z.is_finite().then(|| sigmoid(z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_predictor_matches_validated_model_and_rejects_bad_frames() {
        let examples: Vec<_> = (0..80)
            .map(|i| OwnershipTrainingExample {
                features: vec![
                    if i < 40 {
                        0.05 + f64::from(i % 7) * 0.01
                    } else {
                        0.8 + f64::from(i % 7) * 0.01
                    },
                    0.5,
                ],
                owns_speech: i >= 40,
            })
            .collect();
        let model = OwnershipModel::train(
            vec!["activity".into(), "constant".into()],
            &examples,
            OwnershipTrainingSettings::default(),
            None,
        )
        .unwrap();
        let predictor = model.predictor().unwrap();
        for i in 0..100 {
            let features = [f64::from(i) / 100.0, 0.5];
            assert!(
                (model.estimate(&features).unwrap() - predictor.estimate(&features).unwrap()).abs()
                    < 1e-14
            );
        }
        assert!(predictor.estimate(&[f64::NAN, 0.5]).is_none());
        assert!(predictor.estimate(&[0.5]).is_none());
    }
}
fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + (-z).exp())
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}
