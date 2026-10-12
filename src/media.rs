//! Strict native decoding. Decoder errors never become skipped packets or silence.
use crate::{RenderBlocker, SourceMetadata, SourceTimebase};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::atomic::AtomicBool,
};
use symphonia::core::{
    codecs::audio::{AudioDecoderOptions, well_known::*},
    errors::Error,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

pub(crate) fn decode(
    path: &Path,
    cancelled: Option<&AtomicBool>,
    consume: impl FnMut(u32, &[f32]) -> Result<(), RenderBlocker>,
) -> Result<SourceMetadata, RenderBlocker> {
    decode_channel(path, None, cancelled, consume)
}

pub(crate) fn decode_channel(
    path: &Path,
    channel: Option<u16>,
    cancelled: Option<&AtomicBool>,
    mut consume: impl FnMut(u32, &[f32]) -> Result<(), RenderBlocker>,
) -> Result<SourceMetadata, RenderBlocker> {
    decode_channels(path, &[channel], cancelled, |rate, channels| {
        consume(rate, &channels[0])
    })
}

pub(crate) fn decode_channels(
    path: &Path,
    selected_channels: &[Option<u16>],
    cancelled: Option<&AtomicBool>,
    mut consume: impl FnMut(u32, &[Vec<f32>]) -> Result<(), RenderBlocker>,
) -> Result<SourceMetadata, RenderBlocker> {
    crate::prepared_analysis::check_cancelled(cancelled)?;
    let aiff_frames = aiff_frame_count(path)?;
    let source = MediaSourceStream::new(Box::new(File::open(path)?), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }
    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            source,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|error| failure(path, error))?;
    if format
        .tracks()
        .iter()
        .filter(|track| track.track_type() == Some(TrackType::Audio))
        .count()
        != 1
    {
        return Err(RenderBlocker(
            "requires exactly one mono audio stream; multiple audio streams are ambiguous".into(),
        ));
    }
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| RenderBlocker("no supported audio track".into()))?;
    let parameters = track
        .codec_params
        .as_ref()
        .and_then(|parameters| parameters.audio())
        .cloned()
        .ok_or_else(|| RenderBlocker("no audio codec parameters".into()))?;
    let declared_frames = track.num_frames;
    if !matches!(
        parameters.codec,
        CODEC_ID_AAC
            | CODEC_ID_ALAC
            | CODEC_ID_FLAC
            | CODEC_ID_MP3
            | CODEC_ID_VORBIS
            | CODEC_ID_PCM_S8
            | CODEC_ID_PCM_S16BE
            | CODEC_ID_PCM_S24BE
            | CODEC_ID_PCM_S32BE
            | CODEC_ID_PCM_F64LE
    ) {
        return Err(RenderBlocker("unsupported audio codec profile".into()));
    }
    if parameters.codec == CODEC_ID_AAC
        && parameters
            .extra_data
            .as_ref()
            .is_none_or(|data| data.first().is_none_or(|first| first >> 3 != 2))
    {
        return Err(RenderBlocker(
            "only AAC-LC is supported; HE-AAC is unsupported".into(),
        ));
    }
    let track_id = track.id;
    let rate = parameters
        .sample_rate
        .ok_or_else(|| RenderBlocker("audio sample rate is missing".into()))?;
    let channels = parameters
        .channels
        .as_ref()
        .map_or(0, |channels| channels.count());
    if channels == 0
        || selected_channels.iter().any(|channel| {
            usize::from(channel.unwrap_or(0)) >= channels || (channel.is_none() && channels != 1)
        })
    {
        return Err(RenderBlocker(format!(
            "{}: requires actual mono audio",
            path.display()
        )));
    }
    SourceTimebase::new(rate, 0)?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(
            &parameters,
            &AudioDecoderOptions::default().verify(true).gapless(true),
        )
        .map_err(|error| failure(path, error))?;
    let mut frame_count = 0_u64;
    loop {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(error) => return Err(failure(path, error)),
        };
        if packet.track_id != track_id {
            continue;
        }
        let decoded = decoder
            .decode(&packet)
            .map_err(|error| failure(path, error))?;
        if decoded.spec().rate() != rate || decoded.spec().channels().count() != channels {
            return Err(RenderBlocker(
                "audio rate/channels change within the source".into(),
            ));
        }
        let mut buffer = Vec::<f32>::new();
        buffer
            .try_reserve_exact(decoded.frames())
            .map_err(|_| RenderBlocker("not enough memory to decode audio packet".into()))?;
        decoded.copy_to_vec_interleaved(&mut buffer);
        let selected: Vec<Vec<_>> = selected_channels
            .iter()
            .map(|channel| {
                buffer
                    .chunks_exact(channels)
                    .map(|frame| frame[usize::from(channel.unwrap_or(0))])
                    .collect()
            })
            .collect();
        if selected.iter().flatten().any(|sample| !sample.is_finite()) {
            return Err(RenderBlocker("audio contains invalid samples".into()));
        }
        frame_count = frame_count
            .checked_add(decoded.frames() as u64)
            .ok_or_else(|| RenderBlocker("decoded frame count overflow".into()))?;
        consume(rate, &selected)?;
    }
    // AAC-LC does not guarantee gapless timing. Retain every decoded frame; session validation
    // compares that duration and requires the supplied-alignment declaration, never padding/trim.
    let expected = if parameters.codec == CODEC_ID_AAC {
        None
    } else {
        aiff_frames.or(declared_frames)
    };
    if frame_count == 0 || expected.is_some_and(|expected| expected != frame_count) {
        return Err(RenderBlocker(format!(
            "{}: truncated audio or decoded length disagrees with metadata (decoded {frame_count}, expected {expected:?})",
            path.display()
        )));
    }
    if decoder.finalize().verify_ok == Some(false) {
        return Err(RenderBlocker(
            "decoded audio failed integrity verification".into(),
        ));
    }
    Ok(SourceMetadata {
        grid: SourceTimebase::new(rate, frame_count)?,
        channels: 1,
        timing_origin: None,
        gapless_timing: parameters.codec != CODEC_ID_AAC,
    })
}

fn failure(path: &Path, error: Error) -> RenderBlocker {
    RenderBlocker(format!(
        "{}: unsupported, corrupt or truncated audio: {error}",
        path.display()
    ))
}

fn aiff_frame_count(path: &Path) -> Result<Option<u64>, RenderBlocker> {
    let mut file = File::open(path)?;
    let mut header = [0_u8; 12];
    file.read_exact(&mut header)?;
    if &header[..4] != b"FORM" {
        return Ok(None);
    }
    if &header[8..] != b"AIFF" && &header[8..] != b"AIFC" {
        return Err(RenderBlocker("unsupported FORM audio profile".into()));
    }
    let end = u64::from(u32::from_be_bytes(header[4..8].try_into().unwrap())) + 8;
    if end != file.metadata()?.len() {
        return Err(RenderBlocker(
            "AIFF container is truncated or has conflicting size".into(),
        ));
    }
    let mut frames = None;
    let mut frame_bytes = None;
    let mut data_bytes = None;
    while file.stream_position()? < end {
        let mut chunk = [0_u8; 8];
        file.read_exact(&mut chunk)?;
        let length = u64::from(u32::from_be_bytes(chunk[4..].try_into().unwrap()));
        let next = file
            .stream_position()?
            .checked_add(length + length % 2)
            .filter(|&position| position <= end)
            .ok_or_else(|| RenderBlocker("truncated AIFF chunk".into()))?;
        if &chunk[..4] == b"COMM" {
            if length < 18 || frames.is_some() {
                return Err(RenderBlocker("invalid AIFF COMM metadata".into()));
            }
            let mut common = [0_u8; 18];
            file.read_exact(&mut common)?;
            if u16::from_be_bytes(common[..2].try_into().unwrap()) != 1 {
                return Err(RenderBlocker("AIFF requires actual mono audio".into()));
            }
            let bits = u16::from_be_bytes(common[6..8].try_into().unwrap());
            if !matches!(bits, 8 | 16 | 24 | 32) {
                return Err(RenderBlocker("unsupported AIFF PCM sample size".into()));
            }
            frame_bytes = Some(u64::from(bits / 8));
            frames = Some(u64::from(u32::from_be_bytes(
                common[2..6].try_into().unwrap(),
            )));
        } else if &chunk[..4] == b"SSND" {
            if length < 8 || data_bytes.is_some() {
                return Err(RenderBlocker("invalid AIFF SSND data".into()));
            }
            let mut sound = [0_u8; 8];
            file.read_exact(&mut sound)?;
            if sound != [0; 8] {
                return Err(RenderBlocker(
                    "unsupported block-aligned AIFF profile".into(),
                ));
            }
            data_bytes = Some(length - 8);
        }
        file.seek(SeekFrom::Start(next))?;
    }
    let frames = frames.ok_or_else(|| RenderBlocker("AIFF has no frame-count metadata".into()))?;
    // Validate declared PCM size independently so a decoder EOF cannot hide truncation.
    if frames.checked_mul(frame_bytes.unwrap_or(0)) != data_bytes {
        return Err(RenderBlocker(
            "AIFF frame count disagrees with SSND data length".into(),
        ));
    }
    Ok(Some(frames))
}
