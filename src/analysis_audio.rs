//! Bounded native decoding into verified, disk-backed analysis-only copies.
use crate::{
    RenderBlocker, SourceMetadata, audio_io::decode_chunks, prepared_analysis::check_cancelled,
};
use rubato::{FftFixedIn, Resampler};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

pub const ANALYSIS_SAMPLE_RATE: u32 = 16_000;
pub const AUDIO_DECODER_RECIPE: &str =
    "bwavfile-2.0.1-pcm-f32/symphonia-0.6.1-mono-f64-to-f32-timing-v3";
pub const ANALYSIS_RESAMPLER_RECIPE: &str =
    "rubato-0.16.2-fft-fixed-in-checked-rational-pow2-max4096-sub2-delay-ceil-tail-v2";
static STAGING_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AnalysisAudioIdentity {
    pub source_fingerprint: String,
    pub source: SourceMetadata,
    pub source_time_reference: Option<u64>,
    pub decoder_recipe: String,
    pub resampler_recipe: String,
    pub sample_rate: u32,
    pub frame_count: u64,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Manifest {
    identity: AnalysisAudioIdentity,
    audio_sha256: String,
}

pub struct PreparedAnalysisAudio {
    pub identity: AnalysisAudioIdentity,
    path: PathBuf,
}
impl PreparedAnalysisAudio {
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Bounded reads use the artifact's explicit rate (16 kHz for analysis,
    /// selected device rate for audition), never an implicit native grid.
    pub fn read_frames(&self, range: Range<usize>) -> Result<Vec<f32>, RenderBlocker> {
        if range.start > range.end
            || range.len() > 65_536
            || range.end as u64 > self.identity.frame_count
        {
            return Err(RenderBlocker(
                "analysis read is outside the artifact or exceeds 65,536 frames".into(),
            ));
        }
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(
            (range.start as u64)
                .checked_mul(4)
                .ok_or_else(|| RenderBlocker("analysis offset overflow".into()))?,
        ))?;
        let mut bytes = vec![0; range.len() * 4];
        file.read_exact(&mut bytes)?;
        let samples: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(RenderBlocker(
                "analysis cache contains invalid samples".into(),
            ));
        }
        Ok(samples)
    }
}

pub fn prepare_analysis_audio(
    path: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedAnalysisAudio, RenderBlocker> {
    prepare_analysis_audio_in(
        path,
        &std::env::temp_dir().join("cleanslate-analysis-audio-v1"),
        cancelled,
    )
}

pub fn prepare_analysis_audio_in(
    path: &Path,
    cache: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedAnalysisAudio, RenderBlocker> {
    prepare_audio_at_rate_in(path, cache, ANALYSIS_SAMPLE_RATE, cancelled)
}

pub(crate) fn prepare_audio_at_rate_in(
    path: &Path,
    cache: &Path,
    output_rate: u32,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedAnalysisAudio, RenderBlocker> {
    if output_rate == 0 {
        return Err(RenderBlocker("playback rate must be nonzero".into()));
    }
    check_cancelled(cancelled)?;
    let fingerprint = crate::project::fingerprint_cancellable(path, cancelled)?;
    let key = format!(
        "{:x}",
        Sha256::digest(
            format!(
                "{fingerprint}/{AUDIO_DECODER_RECIPE}/{ANALYSIS_RESAMPLER_RECIPE}{}",
                if output_rate == ANALYSIS_SAMPLE_RATE {
                    String::new()
                } else {
                    format!("/output-{output_rate}")
                }
            )
            .as_bytes()
        )
    );
    let destination = cache.join(key);
    if let Some(audio) = read_cached(&destination, &fingerprint, output_rate, cancelled)? {
        return Ok(audio);
    }
    check_cancelled(cancelled)?;
    fs::create_dir_all(cache)?;
    let nonce = STAGING_ID.fetch_add(1, Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = Staging(cache.join(format!(".audio-{}-{stamp}-{nonce}.tmp", std::process::id())));
    fs::create_dir(&staging.0)?;
    let mut writer = Some(BufWriter::new(File::create(staging.0.join("audio.f32"))?));
    let mut converter = None;
    let decoded = decode_chunks(path, cancelled, |rate, packet| {
        if converter.is_none() {
            converter = Some(StreamResampler::new(
                rate,
                output_rate,
                writer.take().unwrap(),
            )?);
        }
        converter.as_mut().unwrap().feed(packet, cancelled)
    })?;
    let mut converter =
        converter.ok_or_else(|| RenderBlocker("source contains no audio".into()))?;
    converter.finish(cancelled)?;
    if converter.native_count != decoded.metadata.grid.frame_count {
        return Err(RenderBlocker(
            "decoded source count changed during analysis preparation".into(),
        ));
    }
    let identity = AnalysisAudioIdentity {
        source_fingerprint: fingerprint.clone(),
        source: decoded.metadata,
        source_time_reference: decoded
            .bext
            .as_ref()
            .map(|bext| {
                crate::SourceTimebase::new(decoded.format.sample_rate, bext.time_reference)
                    .and_then(|grid| grid.duration())
            })
            .transpose()?,
        decoder_recipe: AUDIO_DECODER_RECIPE.into(),
        resampler_recipe: ANALYSIS_RESAMPLER_RECIPE.into(),
        sample_rate: output_rate,
        frame_count: converter.written,
    };
    let audio_sha256 = format!("{:x}", converter.hash.finalize());
    drop(converter.writer);
    check_cancelled(cancelled)?;
    if crate::project::fingerprint_cancellable(path, cancelled)? != fingerprint {
        return Err(RenderBlocker(
            "source changed during analysis audio preparation".into(),
        ));
    }
    let manifest = serde_json::to_vec(&Manifest {
        identity,
        audio_sha256,
    })
    .map_err(|error| RenderBlocker(error.to_string()))?;
    let mut metadata = File::create(staging.0.join("identity.json"))?;
    metadata.write_all(&manifest)?;
    metadata.sync_all()?;
    drop(metadata);
    check_cancelled(cancelled)?;
    // Install only a complete directory. A cancelled/failed job never replaces accepted evidence.
    if destination.exists() {
        if let Some(audio) = read_cached(&destination, &fingerprint, output_rate, cancelled)? {
            return Ok(audio);
        }
        check_cancelled(cancelled)?;
        // Keep proven-invalid contents recoverable; cancellation and ordinary I/O
        // errors propagate before mutation, and accepted artifacts are never deleted.
        let invalid = cache.join(format!(".invalid-{}-{stamp}-{nonce}", std::process::id()));
        fs::rename(&destination, invalid)?;
    }
    fs::rename(&staging.0, &destination)?;
    read_cached(&destination, &fingerprint, output_rate, cancelled)?
        .ok_or_else(|| RenderBlocker("new analysis artifact failed integrity verification".into()))
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_cached(
    directory: &Path,
    source: &str,
    output_rate: u32,
    cancelled: Option<&AtomicBool>,
) -> Result<Option<PreparedAnalysisAudio>, RenderBlocker> {
    check_cancelled(cancelled)?;
    let mut bytes = Vec::new();
    let file = match File::open(directory.join("identity.json")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    file.take(65_537).read_to_end(&mut bytes)?;
    if bytes.len() > 65_536 {
        return Ok(None);
    }
    let manifest: Manifest = match serde_json::from_slice(&bytes) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(None),
    };
    let id = &manifest.identity;
    let expected = match analysis_length(
        id.source.grid.frame_count,
        id.source.grid.sample_rate,
        output_rate,
    ) {
        Ok(expected) => expected,
        Err(_) => return Ok(None),
    };
    let path = directory.join("audio.f32");
    let audio = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let stored_bytes = audio.metadata()?.len();
    drop(audio);
    if id.source_fingerprint != source
        || id.decoder_recipe != AUDIO_DECODER_RECIPE
        || id.resampler_recipe != ANALYSIS_RESAMPLER_RECIPE
        || id.sample_rate != output_rate
        || id.source.channels != 1
        || id.frame_count != expected
        || expected == 0
        || stored_bytes
            != expected
                .checked_mul(4)
                .ok_or_else(|| RenderBlocker("analysis length overflow".into()))?
        || crate::project::fingerprint_cancellable(&path, cancelled)? != manifest.audio_sha256
    {
        return Ok(None);
    }
    check_cancelled(cancelled)?;
    Ok(Some(PreparedAnalysisAudio {
        identity: manifest.identity,
        path,
    }))
}

fn analysis_length(frames: u64, rate: u32, output_rate: u32) -> Result<u64, RenderBlocker> {
    if rate == 0 {
        return Err(RenderBlocker("audio rate must be nonzero".into()));
    }
    u64::try_from((u128::from(frames) * u128::from(output_rate)).div_ceil(u128::from(rate)))
        .map_err(|_| RenderBlocker("analysis frame count overflow".into()))
}

fn conversion_chunk(rate: u32, output_rate: u32) -> Result<(usize, usize), RenderBlocker> {
    if rate == 0 {
        return Err(RenderBlocker("audio rate must be nonzero".into()));
    }
    // Match pinned Rubato 0.16.2's gcd/subchunk geometry before it allocates.
    // Bound the FFT, not the nominal source rate: 7 kHz and 768 kHz can both be safe.
    const MAX_FFT_INPUT: usize = 1_048_576;
    const MAX_FFT_OUTPUT: usize = 65_536;
    let mut a = rate;
    let mut b = output_rate;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    if a == 0 {
        return Err(RenderBlocker("audio rate must be nonzero".into()));
    }
    let input = (rate / a) as usize;
    let output = (output_rate / a) as usize;
    let desired = (rate as usize / 8).clamp(1, 2048);
    let maximum = (desired / input)
        .max(1)
        .min(MAX_FFT_INPUT / input)
        .min(MAX_FFT_OUTPUT / output);
    if maximum == 0 {
        return Err(RenderBlocker("unsupported conversion FFT geometry: input exceeds 1,048,576 or output exceeds 65,536 frames".into()));
    }
    // Powers of two avoid accidentally choosing a large prime subchunk at common rates.
    let factor = 1usize << (usize::BITS - 1 - maximum.leading_zeros());
    let fft_input = input
        .checked_mul(factor)
        .ok_or_else(|| RenderBlocker("conversion FFT geometry overflow".into()))?;
    let fft_output = output
        .checked_mul(factor)
        .ok_or_else(|| RenderBlocker("conversion FFT geometry overflow".into()))?;
    let chunk = fft_input
        .checked_mul(2)
        .ok_or_else(|| RenderBlocker("conversion chunk overflow".into()))?
        .min(4096);
    let available = (fft_input - 1)
        .checked_add(chunk)
        .ok_or_else(|| RenderBlocker("conversion buffer overflow".into()))?;
    let output_max = (available / fft_input)
        .checked_mul(fft_output)
        .filter(|&frames| frames > 0 && frames <= 131_072)
        .ok_or_else(|| RenderBlocker("unsupported conversion output allocation".into()))?;
    Ok((chunk, output_max))
}

struct StreamResampler {
    rate: u32,
    output_rate: u32,
    converter: Option<FftFixedIn<f32>>,
    input: Vec<f32>,
    filled: usize,
    output: Vec<Vec<f32>>,
    skip: usize,
    native_count: u64,
    written: u64,
    writer: BufWriter<File>,
    hash: Sha256,
}
impl StreamResampler {
    fn new(rate: u32, output_rate: u32, writer: BufWriter<File>) -> Result<Self, RenderBlocker> {
        analysis_length(1, rate, output_rate)?;
        let converter = if rate == output_rate {
            None
        } else {
            let (chunk, output_max) = conversion_chunk(rate, output_rate)?;
            let converter =
                FftFixedIn::<f32>::new(rate as usize, output_rate as usize, chunk, 2, 1).map_err(
                    |error| RenderBlocker(format!("analysis resampler could not start: {error}")),
                )?;
            if converter.input_frames_max() > 4096 || converter.output_frames_max() > output_max {
                return Err(RenderBlocker(
                    "resampler geometry disagrees with checked allocation bounds".into(),
                ));
            }
            Some(converter)
        };
        let skip = converter.as_ref().map_or(0, Resampler::output_delay);
        let input = vec![0.0; converter.as_ref().map_or(0, Resampler::input_frames_max)];
        let output = vec![vec![
            0.0;
            converter
                .as_ref()
                .map_or(0, Resampler::output_frames_max)
        ]];
        Ok(Self {
            rate,
            output_rate,
            converter,
            input,
            filled: 0,
            output,
            skip,
            native_count: 0,
            written: 0,
            writer,
            hash: Sha256::new(),
        })
    }
    fn feed(
        &mut self,
        mut samples: &[f32],
        cancelled: Option<&AtomicBool>,
    ) -> Result<(), RenderBlocker> {
        self.native_count = self
            .native_count
            .checked_add(samples.len() as u64)
            .ok_or_else(|| RenderBlocker("native count overflow".into()))?;
        if self.converter.is_none() {
            return self.write(samples);
        }
        while !samples.is_empty() {
            check_cancelled(cancelled)?;
            let needed = self.converter.as_ref().unwrap().input_frames_next();
            let count = (needed - self.filled).min(samples.len());
            self.input[self.filled..self.filled + count].copy_from_slice(&samples[..count]);
            self.filled += count;
            samples = &samples[count..];
            if self.filled == needed {
                self.process(u64::MAX)?;
                self.filled = 0;
            }
        }
        Ok(())
    }
    fn process(&mut self, limit: u64) -> Result<(), RenderBlocker> {
        let converter = self.converter.as_mut().unwrap();
        let count = converter.input_frames_next();
        let (_, produced) = converter
            .process_into_buffer(&[&self.input[..count]], &mut self.output, None)
            .map_err(|error| RenderBlocker(format!("analysis resampling failed: {error}")))?;
        let skip = self.skip.min(produced);
        self.skip -= skip;
        let count = (produced - skip)
            .min(usize::try_from(limit.saturating_sub(self.written)).unwrap_or(usize::MAX));
        write_samples(
            &mut self.writer,
            &mut self.hash,
            &self.output[0][skip..skip + count],
        )?;
        self.written += count as u64;
        Ok(())
    }
    fn write(&mut self, samples: &[f32]) -> Result<(), RenderBlocker> {
        write_samples(&mut self.writer, &mut self.hash, samples)?;
        self.written += samples.len() as u64;
        Ok(())
    }
    fn finish(&mut self, cancelled: Option<&AtomicBool>) -> Result<(), RenderBlocker> {
        let length = analysis_length(self.native_count, self.rate, self.output_rate)?;
        while self.written < length {
            check_cancelled(cancelled)?;
            self.input[self.filled..].fill(0.0);
            self.process(length)?;
            self.filled = 0;
        }
        if self.written != length {
            return Err(RenderBlocker(
                "resampler produced inconsistent analysis duration".into(),
            ));
        }
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        check_cancelled(cancelled)
    }
}

fn write_samples(
    writer: &mut BufWriter<File>,
    hash: &mut Sha256,
    samples: &[f32],
) -> Result<(), RenderBlocker> {
    // Fixed 16 KiB encoding buffer; hash/write in bounded blocks even for two-hour sources.
    let mut bytes = [0_u8; 4096 * 4];
    for chunk in samples.chunks(4096) {
        for (index, sample) in chunk.iter().enumerate() {
            if !sample.is_finite() {
                return Err(RenderBlocker(
                    "analysis resampling produced invalid samples".into(),
                ));
            }
            bytes[index * 4..index * 4 + 4].copy_from_slice(&sample.to_le_bytes());
        }
        let encoded = &bytes[..chunk.len() * 4];
        writer.write_all(encoded)?;
        hash.update(encoded);
    }
    Ok(())
}
