//! Ticket 92 feasibility only: synchronous, allocating inference, never an audio callback.
use ort::{inputs, session::Session, value::Tensor};

pub const MODEL_SHA256: &str = "1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3";
pub const RECIPE: &str = "silero-v5/causal-blackman127-fc7000-delay64/16k512-context64-v1";
const MODEL: &[u8] = include_bytes!("../../../assets/silero_vad.onnx");

/// Immutable evidence, not speaker identity or a gain instruction. A reset starts a new epoch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub epoch: u64,
    pub end_16k: u64,
    pub available_at: u64,
    pub probability: f32,
}

pub struct Detector {
    session: Session,
    rate: u32,
    taps: Vec<[f32; 127]>,
    history: [f32; 256],
    seen: u64,
    produced: u64,
    frame: Vec<f32>,
    context: [f32; 64],
    state: Vec<f32>,
    epoch: u64,
    fault: bool,
}

impl Detector {
    pub fn new(rate: u32) -> Result<Self, String> {
        if ![44_100, 48_000, 96_000].contains(&rate) {
            return Err("unsupported native sample rate".into());
        }
        let session = Session::builder()
            .and_then(|b| b.with_intra_threads(1))
            .and_then(|b| b.with_inter_threads(1))
            .and_then(|b| b.commit_from_memory(MODEL))
            .map_err(|e| e.to_string())?;
        // Rational phases are prepared off the audio thread; no trigonometry per sample.
        let taps = (0..160)
            .map(|phase| {
                let fractional = f64::from((phase * rate) % 16_000) / 16_000.0;
                let mut taps = std::array::from_fn(|i| {
                    let x = i as f64 - 63.0 - fractional;
                    let fc = 7_000.0 / f64::from(rate);
                    let sinc = if x.abs() < 1e-12 {
                        2.0 * fc
                    } else {
                        (std::f64::consts::TAU * fc * x).sin() / (std::f64::consts::PI * x)
                    };
                    let w = std::f64::consts::TAU * i as f64 / 126.0;
                    (sinc * (0.42 - 0.5 * w.cos() + 0.08 * (2.0 * w).cos())) as f32
                });
                let sum: f32 = taps.iter().sum();
                taps.iter_mut().for_each(|t| *t /= sum);
                taps
            })
            .collect();
        Ok(Self {
            session,
            rate,
            taps,
            history: [0.0; 256],
            seen: 0,
            produced: 0,
            frame: Vec::with_capacity(512),
            context: [0.0; 64],
            state: vec![0.0; 256],
            epoch: 0,
            fault: false,
        })
    }

    pub fn reset(&mut self) {
        self.history.fill(0.0);
        self.seen = 0;
        self.produced = 0;
        self.frame.clear();
        self.context.fill(0.0);
        self.state.fill(0.0);
        self.epoch += 1;
        self.fault = false;
    }

    pub fn push(&mut self, samples: &[f32], mut emit: impl FnMut(Event)) -> Result<(), String> {
        if self.fault {
            return Err("detector fault requires reset".into());
        }
        if samples.iter().any(|s| !s.is_finite()) {
            self.fault = true;
            return Err("nonfinite input".into());
        }
        for &sample in samples {
            self.history[self.seen as usize % 256] = sample;
            self.seen += 1;
            if self.seen * 16_000 / u64::from(self.rate) <= self.produced {
                continue;
            }
            self.produced += 1;
            let center = (self.produced * u64::from(self.rate) / 16_000) as i64 - 64;
            let mut value = 0.0;
            for (i, tap) in self.taps[self.produced as usize % 160].iter().enumerate() {
                let index = center + i as i64 - 63;
                if index >= 0 {
                    value += tap * self.history[index as usize % 256];
                }
            }
            self.frame.push(value);
            if self.frame.len() == 512 {
                let probability = match self.infer() {
                    Ok(p) => p,
                    Err(e) => {
                        self.fault = true;
                        return Err(e);
                    }
                };
                emit(Event {
                    epoch: self.epoch,
                    end_16k: self.produced,
                    available_at: self.seen,
                    probability,
                });
                self.frame.clear();
            }
        }
        Ok(())
    }

    fn infer(&mut self) -> Result<f32, String> {
        let mut run = || -> ort::Result<(f32, Vec<f32>)> {
            let mut input = self.context.to_vec();
            input.extend_from_slice(&self.frame);
            let outputs = self.session.run(inputs![
                "input" => Tensor::from_array(([1, 576], input))?,
                "state" => Tensor::from_array(([2, 1, 128], self.state.clone()))?,
                "sr" => Tensor::from_array((Vec::<usize>::new(), vec![16_000_i64]))?
            ])?;
            Ok((
                outputs["output"].try_extract_tensor::<f32>()?.1[0],
                outputs["stateN"].try_extract_tensor::<f32>()?.1.to_vec(),
            ))
        };
        let (p, state) = run().map_err(|e| e.to_string())?;
        if !p.is_finite() || !(0.0..=1.0).contains(&p) || state.iter().any(|s| !s.is_finite()) {
            return Err("invalid model result".into());
        }
        self.state = state;
        self.context.copy_from_slice(&self.frame[448..]);
        Ok(p)
    }
}
