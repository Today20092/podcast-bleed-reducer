use crate::{RenderBlocker, audio_io::Stem};
use iced::{Color, Point, Rectangle, Renderer, Theme, mouse, widget::canvas};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const BASE_BUCKET: usize = 256;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WaveformCache {
    pub sample_count: usize,
    levels: Vec<WaveformLevel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WaveformLevel {
    samples_per_bucket: usize,
    peaks: Vec<(f32, f32)>,
}

#[derive(Deserialize, Serialize)]
struct PersistedWaveforms {
    sources: Vec<SourceStamp>,
    waveforms: Vec<WaveformCache>,
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct SourceStamp {
    path: PathBuf,
    bytes: u64,
    modified_nanos: u128,
}

impl SourceStamp {
    fn read(path: &Path) -> Result<Self, RenderBlocker> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            path: path.into(),
            bytes: metadata.len(),
            modified_nanos: metadata
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        })
    }
}

impl WaveformCache {
    fn from_samples(samples: &[f32]) -> Self {
        let mut samples_per_bucket = BASE_BUCKET;
        let mut peaks = summarize(samples, samples_per_bucket);
        let mut levels = vec![WaveformLevel {
            samples_per_bucket,
            peaks: peaks.clone(),
        }];
        while peaks.len() > 1 {
            peaks = peaks
                .chunks(2)
                .map(|pair| {
                    pair.iter().fold((1.0_f32, -1.0_f32), |(min, max), peak| {
                        (min.min(peak.0), max.max(peak.1))
                    })
                })
                .collect();
            samples_per_bucket *= 2;
            levels.push(WaveformLevel {
                samples_per_bucket,
                peaks: peaks.clone(),
            });
        }
        Self {
            sample_count: samples.len(),
            levels,
        }
    }

    fn level(&self, samples_per_pixel: usize) -> &WaveformLevel {
        self.levels
            .iter()
            .rev()
            .find(|level| level.samples_per_bucket <= samples_per_pixel.max(BASE_BUCKET))
            .unwrap_or(&self.levels[0])
    }
}

fn summarize(samples: &[f32], bucket: usize) -> Vec<(f32, f32)> {
    samples
        .chunks(bucket)
        .map(|chunk| {
            chunk
                .iter()
                .fold((1.0_f32, -1.0_f32), |(min, max), sample| {
                    (min.min(*sample), max.max(*sample))
                })
        })
        .collect()
}

pub(crate) fn load_or_build(
    project_path: &Path,
    sources: &[PathBuf],
) -> Result<Vec<WaveformCache>, RenderBlocker> {
    let cache_path = project_path.with_extension("waveforms.json");
    let stamps = sources
        .iter()
        .map(|path| SourceStamp::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    if let Ok(bytes) = fs::read(&cache_path)
        && let Ok(cache) = serde_json::from_slice::<PersistedWaveforms>(&bytes)
        && cache.sources == stamps
    {
        return Ok(cache.waveforms);
    }
    let waveforms = sources
        .iter()
        .map(|path| Stem::read(path).map(|stem| WaveformCache::from_samples(&stem.samples)))
        .collect::<Result<Vec<_>, _>>()?;
    fs::write(
        cache_path,
        serde_json::to_vec(&PersistedWaveforms {
            sources: stamps,
            waveforms: waveforms.clone(),
        })
        .map_err(|error| RenderBlocker(error.to_string()))?,
    )?;
    Ok(waveforms)
}

pub(crate) struct Waveform<'a>(pub &'a WaveformCache);

impl<Message> canvas::Program<Message> for Waveform<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let target = (self.0.sample_count as f32 / bounds.width.max(1.0)).ceil() as usize;
        let level = self.0.level(target);
        let center = bounds.height / 2.0;
        let scale = center * 0.9;
        let path = canvas::Path::new(|builder| {
            for (index, (min, max)) in level.peaks.iter().enumerate() {
                let x = index as f32 * bounds.width / level.peaks.len().max(1) as f32;
                builder.move_to(Point::new(x, center - max * scale));
                builder.line_to(Point::new(x, center - min * scale));
            }
        });
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_width(1.0)
                .with_color(Color::from_rgb8(93, 211, 198)),
        );
        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bwavfile::{WaveFmt, WaveWriter};

    fn wav(path: &Path, samples: &[f32]) {
        let writer = WaveWriter::create(path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(samples).unwrap();
        frames.end().unwrap();
    }

    #[test]
    fn cache_preserves_peaks_at_close_and_full_session_levels() {
        let mut samples = vec![0.0; BASE_BUCKET * 8];
        samples[20] = -0.75;
        samples[21] = 0.9;
        let cache = WaveformCache::from_samples(&samples);
        assert_eq!(cache.level(BASE_BUCKET).peaks[0], (-0.75, 0.9));
        let full = cache.level(samples.len());
        assert_eq!(full.peaks, vec![(-0.75, 0.9)]);
    }

    #[test]
    fn persisted_cache_reloads_and_invalidates_with_its_source() {
        let root = std::env::temp_dir().join(format!("cleanslate-waveform-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("voice.wav");
        let project = root.join("session.cleanslate");
        wav(&source, &[0.0, 0.8, -0.6, 0.0]);
        let first = load_or_build(&project, std::slice::from_ref(&source)).unwrap();
        assert!(project.with_extension("waveforms.json").is_file());
        assert_eq!(
            load_or_build(&project, std::slice::from_ref(&source)).unwrap()[0].sample_count,
            4
        );
        fs::remove_file(&source).unwrap();
        wav(&source, &[0.0, 0.2]);
        let changed = load_or_build(&project, &[source]).unwrap();
        assert_eq!(first[0].sample_count, 4);
        assert_eq!(changed[0].sample_count, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "visual evidence uses the local canonical reference session"]
    fn canonical_reference_waveforms_are_detailed_and_aligned() {
        let sources = [
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A01.wav"),
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A02.wav"),
            PathBuf::from(r"C:\Users\User\Downloads\00005603_A03.wav"),
        ];
        assert!(sources.iter().all(|path| path.is_file()));
        let root = std::env::temp_dir().join("cleanslate-canonical-waveform-evidence");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let caches = load_or_build(&root.join("session.cleanslate"), &sources).unwrap();
        assert_eq!(caches.len(), sources.len());
        assert!(
            caches
                .iter()
                .all(|cache| cache.sample_count == caches[0].sample_count)
        );
        assert!(caches.iter().all(|cache| {
            cache
                .level(BASE_BUCKET)
                .peaks
                .iter()
                .any(|(min, max)| *min < 0.0 && *max > 0.0)
        }));
        fs::remove_dir_all(root).unwrap();
    }
}
