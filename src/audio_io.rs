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
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let staging = loop {
            let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = parent.join(format!(
                ".{name}.{}.{nonce}.{sequence}.tmp",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        };
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
        if self.destination.exists() {
            // remove_dir is atomic and refuses a nonempty directory, including a
            // destination filled by another attempt after its initial check.
            fs::remove_dir(&self.destination)?;
        }
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

/// A mono file or one zero-based channel of an interleaved WAV.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioSource {
    pub path: PathBuf,
    pub channel: Option<u16>,
    pub name: Option<String>,
}

impl AudioSource {
    pub fn mono(path: PathBuf) -> Self {
        Self {
            path,
            channel: None,
            name: None,
        }
    }

    pub fn metadata(
        &self,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<SourceMetadata, RenderBlocker> {
        let mut metadata = SourceMetadata::read_cancellable(&self.path, cancelled)?;
        if let Some(channel) = self.channel {
            WaveReader::open(&self.path)?;
            if channel >= metadata.channels {
                return Err(RenderBlocker(
                    "Selected WAV channel is out of range.".into(),
                ));
            }
            metadata.channels = 1;
        }
        Ok(metadata)
    }

    pub(crate) fn fingerprint(
        &self,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<String, std::io::Error> {
        let fingerprint = crate::project::fingerprint_cancellable(&self.path, cancelled)?;
        Ok(self.fingerprint_from_container(&fingerprint))
    }

    pub(crate) fn fingerprint_from_container(&self, fingerprint: &str) -> String {
        use sha2::{Digest, Sha256};
        self.channel.map_or_else(
            || fingerprint.to_owned(),
            |channel| {
                format!(
                    "{:x}",
                    Sha256::digest(format!("{fingerprint}/wav-channel-{channel}").as_bytes())
                )
            },
        )
    }

    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("source")
                .into()
        })
    }
}

/// Recorder labels are optional; malformed or oversized metadata falls back to channel numbers.
pub fn wav_channel_sources(path: &Path) -> Result<Vec<AudioSource>, RenderBlocker> {
    use std::io::{Read, Seek};
    struct MetadataReader {
        file: fs::File,
        remaining: usize,
    }
    impl Read for MetadataReader {
        fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Err(std::io::Error::other("iXML metadata exceeds limit"));
            }
            let size = into.len().min(self.remaining);
            let read = self.file.read(&mut into[..size])?;
            self.remaining -= read;
            Ok(read)
        }
    }
    impl Seek for MetadataReader {
        fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
            self.file.seek(position)
        }
    }
    let mut reader = WaveReader::open(path)?;
    let format = reader.format()?;
    if format.channel_count == 0 {
        return Err(RenderBlocker("WAV contains no channels.".into()));
    }
    let mut names = vec![None; usize::from(format.channel_count)];
    let mut xml = Vec::new();
    if let Ok(mut reader) = WaveReader::new(MetadataReader {
        file: fs::File::open(path)?,
        remaining: 1_048_576,
    }) && reader.read_ixml(&mut xml).is_ok()
        && let Ok(xml) = std::str::from_utf8(&xml)
    {
        names = ixml_channel_names(xml, format.channel_count);
    }
    Ok(names
        .into_iter()
        .enumerate()
        .map(|(channel, name)| AudioSource {
            path: path.into(),
            channel: Some(channel as u16),
            name: Some(name.unwrap_or_else(|| format!("Channel {}", channel + 1))),
        })
        .collect())
}

fn ixml_channel_names(xml: &str, count: u16) -> Vec<Option<String>> {
    let mut names = vec![None; usize::from(count)];
    let value = |body: &str, tag: &str| {
        body.split_once(&format!("<{tag}>"))
            .and_then(|(_, tail)| tail.split_once(&format!("</{tag}>")))
            .map(|(value, _)| value.trim().to_owned())
    };
    let Some(list) = value(xml, "TRACK_LIST") else {
        return names;
    };
    for track in list.split("<TRACK>").skip(1) {
        let Some((track, _)) = track.split_once("</TRACK>") else {
            continue;
        };
        let Some(index) = value(track, "INTERLEAVE_INDEX")
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|i| i.checked_sub(1))
        else {
            continue;
        };
        let Some(slot) = names.get_mut(index) else {
            continue;
        };
        if let Some(name) = value(track, "NAME") {
            let name = name
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&apos;", "'")
                .replace("&amp;", "&");
            if !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control) {
                *slot = Some(name);
            }
        }
    }
    names
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
            format: decoded.format,
            bext: decoded.bext,
            samples,
            metadata: decoded.metadata,
        })
    }
}

pub(crate) struct DecodedMetadata {
    pub metadata: SourceMetadata,
    pub format: WaveFmt,
    pub bext: Option<Bext>,
}

pub(crate) fn render_native(
    source: &Path,
    destination: &Path,
    envelope: &crate::GainEnvelope,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<SourceMetadata, RenderBlocker> {
    render_source_native(
        &AudioSource::mono(source.into()),
        destination,
        envelope,
        cancelled,
    )
}

pub(crate) fn render_source_native(
    source: &AudioSource,
    destination: &Path,
    envelope: &crate::GainEnvelope,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<SourceMetadata, RenderBlocker> {
    render_source_encoded(
        source,
        destination,
        envelope,
        crate::ExportEncoding::PreserveSource,
        cancelled,
    )
}

pub(crate) fn render_source_encoded(
    source: &AudioSource,
    destination: &Path,
    envelope: &crate::GainEnvelope,
    encoding: crate::ExportEncoding,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<SourceMetadata, RenderBlocker> {
    // Validation is bounded; it never retains a native PCM session.
    let decoded = decode_source_chunks(source, cancelled, |_, _| Ok(()))?;
    let mut format = decoded.format;
    if format.common_format() == CommonFormat::IeeeFloatPCM && format.bits_per_sample == 64 {
        format = WaveFmt::new_pcm_mono(format.sample_rate, 32);
        format.tag = 3;
    }
    format = encoding.format(format);
    // Private float intermediates may use the writer's automatic RF64 promotion;
    // the selected final delivery, not its larger intermediate, must fit RIFF.
    if encoding != crate::ExportEncoding::Float32 {
        crate::export_settings::check_riff_size(
            decoded.metadata.grid.frame_count,
            format,
            decoded.bext.as_ref(),
        )?;
    }
    let mut writer = WaveWriter::create(destination, format)?;
    if let Some(bext) = &decoded.bext {
        writer.write_broadcast_metadata(bext)?;
    }
    let mut writer = writer.audio_frame_writer()?;
    let mut frame = 0_u64;
    let mut output = Vec::with_capacity(65_536);
    let actual = decode_source_chunks(source, cancelled, |_, samples| {
        output.clear();
        for &sample in samples {
            let time = decoded
                .metadata
                .grid
                .time_at_frame(frame, crate::Rounding::Floor)?;
            let sample = sample * envelope.gain_at_time(time);
            if !sample.is_finite() {
                return Err(RenderBlocker("render produced invalid samples".into()));
            }
            output.push(sample);
            frame += 1;
        }
        writer.write_frames(&output)?;
        Ok(())
    })?;
    if actual.metadata != decoded.metadata || frame != decoded.metadata.grid.frame_count {
        return Err(RenderBlocker(
            "source changed during native rendering".into(),
        ));
    }
    writer.end()?;
    Ok(decoded.metadata)
}

pub(crate) fn decode_chunks(
    path: &Path,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    consume: impl FnMut(u32, &[f32]) -> Result<(), RenderBlocker>,
) -> Result<DecodedMetadata, RenderBlocker> {
    decode_source_chunks(&AudioSource::mono(path.into()), cancelled, consume)
}

pub(crate) fn decode_source_chunks(
    source: &AudioSource,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    mut consume: impl FnMut(u32, &[f32]) -> Result<(), RenderBlocker>,
) -> Result<DecodedMetadata, RenderBlocker> {
    decode_recording_chunks(std::slice::from_ref(source), cancelled, |rate, channels| {
        consume(rate, &channels[0])
    })
}

/// Selected WAV channels share one bounded container traversal.
pub(crate) fn decode_recording_chunks(
    sources: &[AudioSource],
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    mut consume: impl FnMut(u32, &[Vec<f32>]) -> Result<(), RenderBlocker>,
) -> Result<DecodedMetadata, RenderBlocker> {
    let source = sources
        .first()
        .ok_or_else(|| RenderBlocker("No channels selected".into()))?;
    if sources.iter().any(|s| s.path != source.path) {
        return Err(RenderBlocker(
            "A recording must use one source container".into(),
        ));
    }
    let path = &source.path;
    crate::prepared_analysis::check_cancelled(cancelled)?;
    let Ok(mut reader) = WaveReader::open(path) else {
        if source.channel.is_some() {
            return Err(RenderBlocker(
                "Channel selection requires a WAV source.".into(),
            ));
        }
        let metadata = crate::media::decode(path, cancelled, |rate, samples| {
            consume(rate, &[samples.to_vec()])
        })?;
        let mut format = WaveFmt::new_pcm_mono(metadata.grid.sample_rate, 32);
        format.tag = 3; // Decoded non-WAV sources export IEEE-float32 without clipping/quantization.
        return Ok(DecodedMetadata {
            metadata,
            format,
            bext: None,
        });
    };
    let native_format = reader.format()?;
    let channels = usize::from(native_format.channel_count);
    let channel = usize::from(source.channel.unwrap_or(0));
    let mut format = native_format;
    if channels == 0
        || sources
            .iter()
            .any(|s| usize::from(s.channel.unwrap_or(0)) >= channels)
        || channel >= channels
        || (source.channel.is_none() && channels != 1)
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
    if source.channel.is_some() {
        format = WaveFmt::new_pcm_mono(native_format.sample_rate, native_format.bits_per_sample);
        if native_format.common_format() == CommonFormat::IeeeFloatPCM {
            format.tag = 3;
        }
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
        let decoded = crate::media::decode_channels(
            path,
            &sources.iter().map(|s| s.channel).collect::<Vec<_>>(),
            cancelled,
            consume,
        )?;
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
    let mut frames = Some(reader.audio_frame_reader()?);
    let block_frames = (65_536 / channels).max(1);
    // Float32 already matches engine samples; avoid a virtual read/conversion per sample.
    let mut raw = if native_format.common_format() == CommonFormat::IeeeFloatPCM
        && native_format.bits_per_sample == 32
    {
        let mut frames = frames.take().unwrap();
        frames.locate(0)?;
        Some(frames.into_inner())
    } else {
        None
    };
    let mut raw_bytes = if raw.is_some() {
        vec![0_u8; block_frames * channels * 4]
    } else {
        vec![]
    };
    let mut buffer = vec![0.0_f32; block_frames * channels];
    let mut selected = vec![Vec::with_capacity(block_frames); sources.len()];
    let mut count = 0_u64;
    while count < frame_count {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let requested = usize::try_from((frame_count - count).min(block_frames as u64)).unwrap();
        let read = if let Some(input) = &mut raw {
            use std::io::Read;
            let bytes = &mut raw_bytes[..requested * channels * 4];
            input.read_exact(bytes).map_err(|error| {
                RenderBlocker(format!(
                    "{}: truncated or unreadable audio: {error}",
                    path.display()
                ))
            })?;
            for (sample, bytes) in buffer.iter_mut().zip(bytes.as_chunks::<4>().0) {
                *sample = f32::from_le_bytes(*bytes);
            }
            requested
        } else {
            frames
                .as_mut()
                .unwrap()
                .read_frames(&mut buffer[..requested * channels])? as usize
        };
        if read == 0 {
            return Err(RenderBlocker(format!(
                "{}: truncated audio",
                path.display()
            )));
        }
        for (samples, source) in selected.iter_mut().zip(sources) {
            samples.clear();
            samples.extend(
                buffer[..read * channels]
                    .chunks_exact(channels)
                    .map(|frame| frame[usize::from(source.channel.unwrap_or(0))]),
            );
        }
        if selected.iter().flatten().any(|sample| !sample.is_finite()) {
            return Err(RenderBlocker(format!(
                "{}: contains invalid samples",
                path.display()
            )));
        }
        consume(format.sample_rate, &selected)?;
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

#[cfg(test)]
pub(crate) mod multichannel_tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    pub(crate) fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "cleanslate-poly-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    pub(crate) fn write_poly(path: &Path, tag: u16, bits: u16, frames: usize) {
        let channels = 3_u16;
        let block = channels * bits / 8;
        let mut format = Vec::new();
        format.extend(tag.to_le_bytes());
        format.extend(channels.to_le_bytes());
        format.extend(48_000_u32.to_le_bytes());
        format.extend((48_000_u32 * u32::from(block)).to_le_bytes());
        format.extend(block.to_le_bytes());
        format.extend(bits.to_le_bytes());
        let xml = b"<BWFXML><TRACK_LIST><TRACK><CHANNEL_INDEX>9</CHANNEL_INDEX><INTERLEAVE_INDEX>3</INTERLEAVE_INDEX><NAME>Tr7</NAME></TRACK><TRACK><INTERLEAVE_INDEX>1</INTERLEAVE_INDEX><NAME>Tr5</NAME></TRACK><TRACK><INTERLEAVE_INDEX>2</INTERLEAVE_INDEX><NAME>Tr6</NAME></TRACK></TRACK_LIST></BWFXML>";
        let mut data = Vec::new();
        for _ in 0..frames {
            for sample in [0.25_f32, -0.5, 0.75] {
                match (tag, bits) {
                    (3, 32) => data.extend(sample.to_le_bytes()),
                    (3, 64) => data.extend(f64::from(sample).to_le_bytes()),
                    (1, 16) => data.extend(((sample * 32768.0) as i16).to_le_bytes()),
                    (1, 24) => data.extend(&((sample * 8388608.0) as i32).to_le_bytes()[..3]),
                    (1, 32) => data.extend(((sample * 2147483648.0) as i32).to_le_bytes()),
                    _ => unreachable!(),
                }
            }
        }
        let mut chunks = Vec::new();
        let mut bext = vec![0_u8; 602];
        bext[338..346].copy_from_slice(&98_158_861_u64.to_le_bytes());
        for (id, body) in [
            (b"fmt ", format.as_slice()),
            (b"bext", bext.as_slice()),
            (b"iXML", xml.as_slice()),
            (b"data", data.as_slice()),
        ] {
            chunks.extend(id);
            chunks.extend((body.len() as u32).to_le_bytes());
            chunks.extend(body);
            if body.len() % 2 != 0 {
                chunks.push(0);
            }
        }
        let mut bytes = b"RIFF".to_vec();
        bytes.extend((chunks.len() as u32 + 4).to_le_bytes());
        bytes.extend(b"WAVE");
        bytes.extend(chunks);
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn selected_channels_decode_across_blocks_and_render_native_mono_for_every_wav_encoding() {
        let root = root();
        for (tag, bits) in [(1, 16), (1, 24), (1, 32), (3, 32), (3, 64)] {
            let path = root.join(format!("poly-{tag}-{bits}.wav"));
            write_poly(&path, tag, bits, 30_001);
            let inputs = wav_channel_sources(&path).unwrap();
            assert_eq!(
                inputs
                    .iter()
                    .map(AudioSource::display_name)
                    .collect::<Vec<_>>(),
                ["Tr5", "Tr6", "Tr7"]
            );
            assert!(decode_chunks(&path, None, |_, _| Ok(())).is_err());
            for (source, expected) in inputs.iter().zip([0.25_f32, -0.5, 0.75]) {
                let mut count = 0;
                let decoded = decode_source_chunks(source, None, |rate, samples| {
                    assert_eq!(rate, 48_000);
                    assert!(samples.len() <= 65_536);
                    assert!(
                        samples
                            .iter()
                            .all(|sample| (*sample - expected).abs() < 0.0001)
                    );
                    count += samples.len();
                    Ok(())
                })
                .unwrap();
                assert_eq!(count, 30_001);
                assert_eq!(decoded.metadata.channels, 1);
                let output =
                    root.join(format!("mono-{tag}-{bits}-{}.wav", source.channel.unwrap()));
                let envelope = crate::GainEnvelope::constant_at_rate(count, 48_000, 0.5).unwrap();
                render_source_native(source, &output, &envelope, None).unwrap();
                assert_eq!(SourceMetadata::read(&output).unwrap(), decoded.metadata);
                assert!(
                    Stem::read(&output)
                        .unwrap()
                        .samples
                        .iter()
                        .all(|sample| (*sample - expected * 0.5).abs() < 0.0001)
                );
            }
            let invalid = AudioSource {
                path: path.clone(),
                channel: Some(3),
                name: None,
            };
            assert!(invalid.metadata(None).is_err());
            assert!(decode_source_chunks(&invalid, None, |_, _| Ok(())).is_err());
            assert!(
                decode_source_chunks(&inputs[0], Some(&AtomicBool::new(true)), |_, _| panic!(
                    "cancelled decode consumed audio"
                ))
                .is_err()
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn channels_have_separate_caches_persistent_names_and_relink_bindings() {
        let root = root();
        let path = root.join("poly.wav");
        write_poly(&path, 3, 32, 48_000);
        let inputs = wav_channel_sources(&path).unwrap();
        let mut project = crate::Project::create_from_sources(
            root.join("session.cleanslate"),
            vec![inputs[2].clone(), inputs[0].clone()],
        )
        .unwrap();
        assert_ne!(
            project.sources[0].fingerprint,
            project.sources[1].fingerprint
        );
        project.require_alignment().unwrap();
        let cache = root.join("audio");
        let a = crate::prepare_source_analysis_audio_in(&inputs[0], &cache, None).unwrap();
        let b = crate::prepare_source_analysis_audio_in(&inputs[2], &cache, None).unwrap();
        assert_ne!(a.path(), b.path());
        assert_ne!(a.identity.source_fingerprint, b.identity.source_fingerprint);
        assert!(
            a.read_frames(200..210)
                .unwrap()
                .iter()
                .all(|s| (*s - 0.25).abs() < 0.001)
        );
        assert!(
            b.read_frames(200..210)
                .unwrap()
                .iter()
                .all(|s| (*s - 0.75).abs() < 0.001)
        );
        let prepared = crate::PreparedAudio::prepare_sources_for_rate(
            &project.audio_sources(),
            None,
            48_000,
            &AtomicBool::new(false),
        )
        .unwrap();
        let packet = prepared.read_frames(99..107).unwrap();
        assert_eq!(packet[0], vec![0.75; 8]);
        assert_eq!(packet[1], vec![0.25; 8]);
        project.rename_track(0, "Alice".into()).unwrap();
        let mut reopened = crate::Project::open(project.path()).unwrap();
        assert_eq!(reopened.sources[0].display_name(), "Alice");
        assert_eq!(reopened.sources[0].channel, Some(2));
        reopened.validate_sources().unwrap();
        let replacement = root.join("moved.wav");
        fs::copy(&path, &replacement).unwrap();
        reopened.relink(0, replacement).unwrap();
        assert_eq!(reopened.sources[0].channel, Some(2));
        assert_eq!(reopened.sources[0].display_name(), "Alice");
        assert!(reopened.pending_source_reviews().is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_or_bad_recorder_names_fall_back_without_using_hardware_channel_indices() {
        assert_eq!(ixml_channel_names("<BWFXML/>", 2), [None, None]);
        assert_eq!(
            ixml_channel_names(
                "<TRACK_LIST><TRACK><CHANNEL_INDEX>1</CHANNEL_INDEX><NAME>Wrong</NAME></TRACK><TRACK><INTERLEAVE_INDEX>2</INTERLEAVE_INDEX><NAME>A &amp; B</NAME></TRACK></TRACK_LIST>",
                2
            ),
            [None, Some("A & B".into())]
        );
    }

    #[test]
    fn nonfinite_selected_samples_and_truncated_wavs_are_rejected() {
        let root = root();
        for bits in [32, 64] {
            let path = root.join(format!("invalid-{bits}.wav"));
            write_poly(&path, 3, bits, 4);
            let mut bytes = fs::read(&path).unwrap();
            let start = bytes.windows(4).position(|chunk| chunk == b"data").unwrap() + 8;
            let width = usize::from(bits / 8);
            if bits == 32 {
                bytes[start + width..start + width * 2].copy_from_slice(&f32::NAN.to_le_bytes());
            } else {
                bytes[start + width..start + width * 2].copy_from_slice(&f64::NAN.to_le_bytes());
            }
            fs::write(&path, &bytes).unwrap();
            let inputs = wav_channel_sources(&path).unwrap();
            assert!(decode_source_chunks(&inputs[1], None, |_, _| Ok(())).is_err());
            decode_source_chunks(&inputs[0], None, |_, _| Ok(())).unwrap();
            bytes.truncate(bytes.len() - width);
            fs::write(&path, bytes).unwrap();
            assert!(decode_source_chunks(&inputs[0], None, |_, _| Ok(())).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
