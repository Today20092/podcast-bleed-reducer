use crate::{RenderBlocker, export_settings::check_riff_size};
use bwavfile::{Bext, CommonFormat, WaveFmt, WaveReader, WaveWriter};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// TPDF in target LSBs. Fixed stream makes retries reproducible and silence exact.
struct Quantizer(u64);
impl Quantizer {
    fn uniform(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1_u64 << 53) as f64
    }
    fn sample(&mut self, sample: f32, bits: u16) -> Result<i32, RenderBlocker> {
        let scale = (1_u64 << (bits - 1)) as f64;
        let value = f64::from(sample) * scale;
        if !sample.is_finite() || value < -scale || value > scale - 1.0 {
            return Err(RenderBlocker("Audio exceeds the integer full-scale range. Choose 32-bit float to retain its levels.".into()));
        }
        if sample == 0.0 {
            return Ok(0);
        }
        let dither = self.uniform() - self.uniform();
        // Only the dither can cross a rail after the undithered overload check.
        let quantized = (value + dither).round().clamp(-scale, scale - 1.0) as i64;
        Ok((quantized << (32 - bits)) as i32)
    }
}

pub(crate) fn encode_delivery(
    inputs: &[PathBuf],
    destination: &Path,
    mut format: WaveFmt,
    bext: Option<&Bext>,
    frames: u64,
    cancelled: Option<&AtomicBool>,
) -> Result<(), RenderBlocker> {
    format.channel_count =
        u16::try_from(inputs.len()).map_err(|_| RenderBlocker("Too many channels.".into()))?;
    format.block_alignment = format
        .channel_count
        .checked_mul(format.bits_per_sample / 8)
        .ok_or_else(|| RenderBlocker("WAV format overflow.".into()))?;
    format.bytes_per_second = format
        .sample_rate
        .checked_mul(u32::from(format.block_alignment))
        .ok_or_else(|| RenderBlocker("WAV format overflow.".into()))?;
    check_riff_size(frames, format, bext)?;
    let readers = inputs.iter().map(|path| {
        let mut reader = WaveReader::open(path)?;
        let input_format = reader.format()?;
        if input_format.channel_count != 1 || input_format.sample_rate != format.sample_rate
            || reader.frame_length()? != frames || input_format.common_format() != CommonFormat::IeeeFloatPCM
            || input_format.bits_per_sample != 32 {
            return Err(RenderBlocker("Prepare a current float32 render before changing delivery encoding; integer renders cannot be dithered twice.".into()));
        }
        Ok(reader)
    }).collect::<Result<Vec<_>, RenderBlocker>>()?;
    let mut readers = readers
        .into_iter()
        .map(|r| r.audio_frame_reader())
        .collect::<Result<Vec<_>, _>>()?;
    let mut writer = WaveWriter::create(destination, format)?;
    if let Some(bext) = bext {
        writer.write_broadcast_metadata(bext)?;
    }
    let mut writer = writer.audio_frame_writer()?;
    let channels = inputs.len();
    let block_frames = 8192;
    let mut mono = vec![0.0_f32; block_frames];
    let mut interleaved = vec![0.0_f32; block_frames * channels];
    let mut integers = Vec::with_capacity(interleaved.len());
    let mut quantizers: Vec<_> = inputs
        .iter()
        .map(|path| {
            Quantizer(
                path.to_string_lossy()
                    .bytes()
                    .fold(0x6a09e667f3bcc909_u64, |seed, b| {
                        seed.wrapping_mul(1099511628211) ^ u64::from(b)
                    })
                    | 1,
            )
        })
        .collect();
    let mut remaining = frames;
    while remaining > 0 {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let count = remaining.min(block_frames as u64) as usize;
        for (channel, reader) in readers.iter_mut().enumerate() {
            let mut read = 0;
            while read < count {
                let n = reader.read_frames(&mut mono[read..count])? as usize;
                if n == 0 {
                    return Err(RenderBlocker("Truncated accepted render.".into()));
                }
                read += n;
            }
            for i in 0..count {
                interleaved[i * channels + channel] = mono[i];
            }
        }
        let samples = &interleaved[..count * channels];
        if format.common_format() == CommonFormat::IntegerPCM {
            integers.clear();
            for (i, &sample) in samples.iter().enumerate() {
                integers.push(quantizers[i % channels].sample(sample, format.bits_per_sample)?);
            }
            writer.write_frames(&integers)?;
        } else {
            if samples.iter().any(|s| !s.is_finite()) {
                return Err(RenderBlocker("Invalid rendered samples.".into()));
            }
            writer.write_frames(samples)?;
        }
        remaining -= count as u64;
    }
    writer.end()?;
    Ok(())
}
