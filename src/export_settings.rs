use crate::{Project, RenderBlocker};
use bwavfile::{Bext, CommonFormat, WaveFmt, WaveReader};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportEncoding {
    #[default]
    PreserveSource,
    Pcm24,
    Float32,
}

impl std::fmt::Display for ExportEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PreserveSource => "Preserve source encoding",
            Self::Pcm24 => "24-bit PCM",
            Self::Float32 => "32-bit float",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExportLayout {
    #[default]
    SeparateMono,
    Multichannel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExportSettings {
    pub layout: ExportLayout,
    pub encoding: ExportEncoding,
}

pub(crate) fn source_format(
    source: &crate::SourceIdentity,
) -> Result<(WaveFmt, Option<Bext>), RenderBlocker> {
    let mut format = WaveFmt::new_pcm_mono(source.metadata.grid.sample_rate, 32);
    format.tag = 3;
    let Ok(mut reader) = WaveReader::open(&source.path) else {
        return Ok((format, None));
    };
    let native = reader.format()?;
    format = WaveFmt::new_pcm_mono(native.sample_rate, native.bits_per_sample);
    if native.common_format() == CommonFormat::IeeeFloatPCM {
        format.tag = 3;
        if native.bits_per_sample == 64 {
            format = WaveFmt::new_pcm_mono(native.sample_rate, 32);
            format.tag = 3;
        }
    }
    Ok((format, reader.broadcast_extension()?))
}

impl ExportEncoding {
    pub(crate) fn format(self, source: WaveFmt) -> WaveFmt {
        match self {
            Self::PreserveSource => source,
            Self::Pcm24 => WaveFmt::new_pcm_mono(source.sample_rate, 24),
            Self::Float32 => {
                let mut format = WaveFmt::new_pcm_mono(source.sample_rate, 32);
                format.tag = 3;
                format
            }
        }
    }
}

impl ExportSettings {
    pub fn validate_layout(&self, project: &Project) -> Result<(), RenderBlocker> {
        if project.sources.is_empty() {
            return Err(RenderBlocker("Import tracks before exporting.".into()));
        }
        if self.layout == ExportLayout::SeparateMono {
            for source in &project.sources {
                let (format, bext) = source_format(source)?;
                check_riff_size(
                    source.metadata.grid.frame_count,
                    self.encoding.format(format),
                    bext.as_ref(),
                )?;
            }
            return Ok(());
        }
        let first = &project.sources[0];
        let (format, bext) = source_format(first)?;
        let format = self.encoding.format(format);
        for source in &project.sources[1..] {
            if source.metadata.grid.sample_rate != first.metadata.grid.sample_rate {
                return Err(RenderBlocker("Multichannel WAV requires matching sample rates. Use separate native-rate WAVs.".into()));
            }
            if source.metadata.grid != first.metadata.grid
                || source.metadata.timing_origin != first.metadata.timing_origin
            {
                return Err(RenderBlocker("Multichannel WAV requires matching frame counts and timing. Use separate WAVs.".into()));
            }
            let (other_format, other_bext) = source_format(source)?;
            let other_format = self.encoding.format(other_format);
            if other_format.common_format() != format.common_format()
                || other_format.bits_per_sample != format.bits_per_sample
            {
                return Err(RenderBlocker("Source encodings differ. Choose 24-bit PCM or 32-bit float for multichannel WAV.".into()));
            }
            if !same_bext(other_bext.as_ref(), bext.as_ref()) {
                return Err(RenderBlocker("Broadcast WAV metadata conflicts. Use separate WAVs to preserve each source's metadata.".into()));
            }
        }
        let mut combined = format;
        combined.block_alignment = combined
            .block_alignment
            .checked_mul(
                u16::try_from(project.sources.len())
                    .map_err(|_| RenderBlocker("Too many channels.".into()))?,
            )
            .ok_or_else(|| RenderBlocker("WAV size overflow.".into()))?;
        check_riff_size(first.metadata.grid.frame_count, combined, bext.as_ref())?;
        Ok(())
    }

    pub fn estimated_audio_bytes(&self, project: &Project) -> Result<u64, RenderBlocker> {
        project.sources.iter().try_fold(0_u64, |total, source| {
            let format = self.encoding.format(source_format(source)?.0);
            source
                .metadata
                .grid
                .frame_count
                .checked_mul(u64::from(format.bits_per_sample / 8))
                .and_then(|bytes| total.checked_add(bytes))
                .ok_or_else(|| RenderBlocker("WAV output size overflow.".into()))
        })
    }
}

fn same_bext(a: Option<&Bext>, b: Option<&Bext>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.description == b.description
                && a.originator == b.originator
                && a.originator_reference == b.originator_reference
                && a.origination_date == b.origination_date
                && a.origination_time == b.origination_time
                && a.time_reference == b.time_reference
                && a.version == b.version
                && a.umid == b.umid
                && a.loudness_value == b.loudness_value
                && a.loudness_range == b.loudness_range
                && a.max_true_peak_level == b.max_true_peak_level
                && a.max_momentary_loudness == b.max_momentary_loudness
                && a.max_short_term_loudness == b.max_short_term_loudness
                && a.coding_history == b.coding_history
        }
        _ => false,
    }
}

pub(crate) fn check_riff_size(
    frames: u64,
    format: WaveFmt,
    bext: Option<&Bext>,
) -> Result<(), RenderBlocker> {
    let metadata = bext.map_or(0, |b| 610 + b.coding_history.len() as u64);
    if frames
        .checked_mul(u64::from(format.block_alignment))
        .and_then(|v| v.checked_add(metadata))
        .and_then(|v| v.checked_add(128))
        .is_none_or(|v| v > u64::from(u32::MAX))
    {
        return Err(RenderBlocker(
            "WAV exceeds the supported 32-bit RIFF size. Use separate WAVs or a shorter recording."
                .into(),
        ));
    }
    Ok(())
}
