use crate::RenderBlocker;
use bwavfile::{Bext, CommonFormat, WaveFmt, WaveReader, WaveWriter};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct AtomicOutput {
    staging: PathBuf,
    destination: PathBuf,
    committed: bool,
}

impl AtomicOutput {
    pub(crate) fn new(destination: &Path) -> Result<Self, RenderBlocker> {
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("rendered");
        let staging = parent.join(format!(".{name}.{}.tmp", std::process::id()));
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir(&staging)?;
        Ok(Self {
            staging,
            destination: destination.into(),
            committed: false,
        })
    }

    pub(crate) fn path(&self, name: &str) -> PathBuf {
        self.staging.join(name)
    }
    pub(crate) fn write_report(&self, report: &str) -> Result<(), RenderBlocker> {
        fs::write(self.path("report.json"), report).map_err(Into::into)
    }
    pub(crate) fn commit(mut self) -> Result<(), RenderBlocker> {
        fs::rename(&self.staging, &self.destination)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for AtomicOutput {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.staging);
        }
    }
}

pub(crate) struct Stem {
    pub path: PathBuf,
    pub format: WaveFmt,
    pub bext: Option<Bext>,
    pub samples: Vec<f32>,
}

impl Stem {
    pub(crate) fn read(path: &Path) -> Result<Self, RenderBlocker> {
        Self::read_cancellable(path, None)
    }

    pub(crate) fn read_cancellable(
        path: &Path,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        let mut reader = WaveReader::open(path)?;
        let format = reader.format()?;
        if format.channel_count != 1
            || format.sample_rate != 48_000
            || !matches!(
                format.common_format(),
                CommonFormat::IntegerPCM | CommonFormat::IeeeFloatPCM
            )
            || !matches!(
                (format.common_format(), format.bits_per_sample),
                (CommonFormat::IntegerPCM, 16 | 24 | 32) | (CommonFormat::IeeeFloatPCM, 32 | 64)
            )
        {
            return Err(RenderBlocker(format!(
                "{}: requires 48 kHz mono PCM16/24/32 or IEEE-float32/64",
                path.display()
            )));
        }
        let frame_count = usize::try_from(reader.frame_length()?)
            .map_err(|_| RenderBlocker("stem is too large for this platform".into()))?;
        let bext = reader.broadcast_extension()?;
        let mut frames = reader.audio_frame_reader()?;
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(frame_count)
            .map_err(|_| RenderBlocker("not enough memory to prepare audio".into()))?;
        let mut buffer = vec![0.0_f32; 65_536];
        while samples.len() < frame_count {
            if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
                return Err(RenderBlocker("audio preparation cancelled".into()));
            }
            let count = buffer.len().min(frame_count - samples.len());
            let read = frames.read_frames(&mut buffer[..count])? as usize;
            if read == 0 {
                return Err(RenderBlocker(format!(
                    "{}: truncated audio",
                    path.display()
                )));
            }
            if buffer[..read].iter().any(|sample| !sample.is_finite()) {
                return Err(RenderBlocker(format!(
                    "{}: contains invalid samples",
                    path.display()
                )));
            }
            samples.extend_from_slice(&buffer[..read]);
        }
        Ok(Self {
            path: path.into(),
            format,
            bext,
            samples,
        })
    }

    pub(crate) fn write(&self, path: &Path, samples: &[f32]) -> Result<(), RenderBlocker> {
        let mut writer = WaveWriter::create(path, self.format)?;
        if let Some(bext) = &self.bext {
            writer.write_broadcast_metadata(bext)?;
        }
        let mut frames = writer.audio_frame_writer()?;
        frames.write_frames(samples)?;
        frames.end()?;
        Ok(())
    }
}

pub(crate) fn waveform(path: &Path, bins: usize) -> Result<Vec<(f32, f32)>, RenderBlocker> {
    let samples = Stem::read(path)?.samples;
    let chunk = samples.len().div_ceil(bins).max(1);
    Ok(samples
        .chunks(chunk)
        .map(|samples| {
            samples
                .iter()
                .fold((0.0_f32, 0.0_f32), |(min, max), sample| {
                    (min.min(*sample), max.max(*sample))
                })
        })
        .collect())
}
