use crate::RenderBlocker;
use crate::SourceTimebase;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceMetadata {
    pub grid: SourceTimebase,
    pub channels: u16,
    #[serde(default)]
    pub timing_origin: Option<u64>,
    #[serde(default)]
    pub gapless_timing: bool,
}

impl SourceMetadata {
    pub fn read(path: &Path) -> Result<Self, RenderBlocker> {
        Self::read_cancellable(path, None)
    }

    pub fn read_cancellable(
        path: &Path,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let Ok(mut reader) = WaveReader::open(path) else {
            return crate::media::decode(path, cancelled, |_, _| Ok(()));
        };
        let format = reader.format()?;
        let bext = reader.broadcast_extension()?;
        Ok(Self {
            grid: SourceTimebase::new(format.sample_rate, reader.frame_length()?)?,
            channels: format.channel_count,
            timing_origin: bext
                .as_ref()
                .map(|bext| {
                    SourceTimebase::new(format.sample_rate, bext.time_reference)
                        .and_then(|grid| grid.duration())
                })
                .transpose()?,
            gapless_timing: true,
        })
    }
}

pub(crate) struct Stem {
    pub path: PathBuf,
    pub format: WaveFmt,
    pub bext: Option<Bext>,
    pub samples: Vec<f32>,
    pub metadata: SourceMetadata,
}

impl Stem {
    pub(crate) fn read(path: &Path) -> Result<Self, RenderBlocker> {
        Self::read_cancellable(path, None)
    }

    pub(crate) fn read_cancellable(
        path: &Path,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Self, RenderBlocker> {
        let mut samples = Vec::new();
        let decoded = decode_chunks(path, cancelled, |_, packet| {
            samples
                .try_reserve(packet.len())
                .map_err(|_| RenderBlocker("not enough memory to prepare audio".into()))?;
            samples.extend_from_slice(packet);
            Ok(())
        })?;
        Ok(Self {
            path: path.into(),
            format: decoded.format,
            bext: decoded.bext,
            samples,
            metadata: decoded.metadata,
        })
    }

    pub(crate) fn write(&self, path: &Path, samples: &[f32]) -> Result<(), RenderBlocker> {
        // The engine is f32. bwavfile cannot write float64 frames; export these
        // decoded sources as explicit native-rate float32 WAV, preserving BEXT.
        let format = if matches!(self.format.common_format(), CommonFormat::IeeeFloatPCM)
            && self.format.bits_per_sample == 64
        {
            let mut format = WaveFmt::new_pcm_mono(self.format.sample_rate, 32);
            format.tag = 3;
            format
        } else {
            self.format
        };
        let mut writer = WaveWriter::create(path, format)?;
        if let Some(bext) = &self.bext {
            writer.write_broadcast_metadata(bext)?;
        }
        let mut frames = writer.audio_frame_writer()?;
        frames.write_frames(samples)?;
        frames.end()?;
        Ok(())
    }
}

pub(crate) struct DecodedMetadata {
    pub metadata: SourceMetadata,
    pub format: WaveFmt,
    pub bext: Option<Bext>,
}

pub(crate) fn decode_chunks(
    path: &Path,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    mut consume: impl FnMut(u32, &[f32]) -> Result<(), RenderBlocker>,
) -> Result<DecodedMetadata, RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancelled)?;
    let Ok(mut reader) = WaveReader::open(path) else {
        let metadata = crate::media::decode(path, cancelled, consume)?;
        let mut format = WaveFmt::new_pcm_mono(metadata.grid.sample_rate, 32);
        format.tag = 3; // Decoded non-WAV sources export IEEE-float32 without clipping/quantization.
        return Ok(DecodedMetadata {
            metadata,
            format,
            bext: None,
        });
    };
    let format = reader.format()?;
    if format.channel_count != 1
        || format.sample_rate == 0
        || !matches!(
            (format.common_format(), format.bits_per_sample),
            (CommonFormat::IntegerPCM, 16 | 24 | 32) | (CommonFormat::IeeeFloatPCM, 32 | 64)
        )
    {
        return Err(RenderBlocker(format!(
            "{}: requires mono PCM16/24/32 or IEEE-float32/64 at a nonzero native rate",
            path.display()
        )));
    }
    let frame_count = reader.frame_length()?;
    let bext = reader.broadcast_extension()?;
    let metadata = SourceMetadata {
        grid: SourceTimebase::new(format.sample_rate, frame_count)?,
        channels: 1,
        timing_origin: bext
            .as_ref()
            .map(|bext| {
                SourceTimebase::new(format.sample_rate, bext.time_reference)
                    .and_then(|grid| grid.duration())
            })
            .transpose()?,
        gapless_timing: true,
    };
    if format.common_format() == CommonFormat::IeeeFloatPCM && format.bits_per_sample == 64 {
        // bwavfile 2.0.1 panics when constructing its float64 frame reader.
        // Retain its native metadata/BEXT, decode samples to the engine's f32 contract.
        let decoded = crate::media::decode(path, cancelled, consume)?;
        if decoded.grid != metadata.grid {
            return Err(RenderBlocker(
                "float64 WAV decoded grid disagrees with native metadata".into(),
            ));
        }
        return Ok(DecodedMetadata {
            metadata,
            format,
            bext,
        });
    }
    let mut frames = reader.audio_frame_reader()?;
    let mut buffer = vec![0.0_f32; 65_536];
    let mut count = 0_u64;
    while count < frame_count {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let requested = usize::try_from((frame_count - count).min(buffer.len() as u64)).unwrap();
        let read = frames.read_frames(&mut buffer[..requested])? as usize;
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
        consume(format.sample_rate, &buffer[..read])?;
        count += read as u64;
    }
    if frame_count == 0 {
        return Err(RenderBlocker("source contains no audio samples".into()));
    }
    Ok(DecodedMetadata {
        metadata,
        format,
        bext,
    })
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
