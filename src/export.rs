use crate::{Project, RenderBlocker, RenderState, audio_io::AtomicOutput, validate};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path, sync::atomic::AtomicBool};

#[derive(Deserialize)]
struct Report {
    output: Output,
}

#[derive(Deserialize)]
struct Output {
    stems: Vec<String>,
}

pub struct PreparedExport {
    output: AtomicOutput,
    project: std::path::PathBuf,
    identity: String,
    files: Vec<(String, String, crate::SourceMetadata)>,
    inputs: Vec<(std::path::PathBuf, String)>,
    layout: crate::ExportLayout,
}

pub fn prepare_export(
    project: &Project,
    destination: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedExport, RenderBlocker> {
    project.validate_sources_cancellable(cancelled)?;
    let plan = delivery_plan(project, cancelled, crate::ExportLayout::SeparateMono)?;
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(RenderBlocker("export destination is not empty".into()));
    }
    let output = AtomicOutput::new(destination)?;
    let mut files = Vec::new();
    let mut inputs = Vec::new();
    for (name, rendered) in plan {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        if fs::metadata(&rendered)?
            .len()
            .checked_sub(8)
            .is_none_or(|bytes| bytes >= u64::from(u32::MAX))
        {
            return Err(RenderBlocker("The private float intermediate exceeds RIFF. Export with 24-bit PCM settings or use a shorter recording.".into()));
        }
        let hash = crate::project::fingerprint_cancellable(&rendered, cancelled)?;
        let metadata = crate::SourceMetadata::read_cancellable(&rendered, cancelled)?;
        {
            use std::io::{Read, Write};
            let mut input = fs::File::open(&rendered)?;
            let mut destination = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output.path(&name))?;
            let mut block = [0; 65_536];
            loop {
                crate::prepared_analysis::check_cancelled(cancelled)?;
                let count = input.read(&mut block)?;
                if count == 0 {
                    break;
                }
                destination.write_all(&block[..count])?;
            }
            destination.sync_all()?;
        }
        if crate::project::fingerprint_cancellable(&output.path(&name), cancelled)? != hash
            || crate::SourceMetadata::read_cancellable(&output.path(&name), cancelled)? != metadata
        {
            return Err(RenderBlocker(
                "Rendered audio changed during the delivery copy.".into(),
            ));
        }
        inputs.push((rendered, hash.clone()));
        files.push((name, hash, metadata));
    }
    crate::prepared_analysis::check_cancelled(cancelled)?;
    Ok(PreparedExport {
        output,
        project: project.path().into(),
        identity: project.delivery_identity()?,
        files,
        inputs,
        layout: crate::ExportLayout::SeparateMono,
    })
}

impl PreparedExport {
    pub fn publish(
        self,
        project: &Project,
        generation: &crate::RenderGeneration,
        expected: u64,
        cancelled: Option<&AtomicBool>,
    ) -> Result<(), RenderBlocker> {
        project.validate_sources_cancellable(cancelled)?;
        delivery_plan(project, cancelled, self.layout)?;
        for (rendered, hash) in &self.inputs {
            if crate::project::fingerprint_cancellable(rendered, cancelled)? != *hash {
                return Err(RenderBlocker(
                    "Accepted rendered audio changed during delivery.".into(),
                ));
            }
        }
        for (name, hash, metadata) in &self.files {
            if crate::project::fingerprint_cancellable(&self.output.path(name), cancelled)? != *hash
                || crate::SourceMetadata::read_cancellable(&self.output.path(name), cancelled)?
                    != *metadata
            {
                return Err(RenderBlocker(
                    "Delivery copies no longer match the accepted render.".into(),
                ));
            }
        }
        let guard = generation
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *guard != expected
            || project.path() != self.project
            || project.delivery_identity()? != self.identity
            || Project::saved_delivery_identity(project.path())? != self.identity
        {
            return Err(RenderBlocker(
                "Delivery became obsolete before publication; previous outputs retained.".into(),
            ));
        }
        project.validate_interactive_sources()?;
        crate::prepared_analysis::check_cancelled(cancelled)?;
        self.output.commit()
    }
}

pub fn prepare_export_with_settings(
    project: &Project,
    destination: &Path,
    settings: crate::ExportSettings,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedExport, RenderBlocker> {
    prepare_export_in_order(
        project,
        destination,
        settings,
        &(0..project.sources.len()).collect::<Vec<_>>(),
        cancelled,
    )
}

pub fn prepare_export_in_order(
    project: &Project,
    destination: &Path,
    settings: crate::ExportSettings,
    source_order: &[usize],
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedExport, RenderBlocker> {
    if source_order.len() != project.sources.len()
        || source_order
            .iter()
            .any(|index| *index >= project.sources.len())
        || source_order.iter().copied().collect::<HashSet<_>>().len() != source_order.len()
    {
        return Err(RenderBlocker(
            "Export order must include every accepted track exactly once.".into(),
        ));
    }
    project.validate_sources_cancellable(cancelled)?;
    settings.validate_layout(project)?;
    let plan = delivery_plan(project, cancelled, settings.layout)?;
    let plan: Vec<_> = source_order
        .iter()
        .map(|index| plan[*index].clone())
        .collect();
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(RenderBlocker("export destination is not empty".into()));
    }
    let output = AtomicOutput::new(destination)?;
    let inputs = plan
        .iter()
        .map(|(_, path)| {
            Ok((
                path.clone(),
                crate::project::fingerprint_cancellable(path, cancelled)?,
            ))
        })
        .collect::<Result<Vec<_>, RenderBlocker>>()?;
    let mut files = Vec::new();
    let groups: Vec<_> = if settings.layout == crate::ExportLayout::Multichannel {
        vec![(
            "session-clean.wav".to_owned(),
            plan.iter().map(|(_, p)| p.clone()).collect::<Vec<_>>(),
            source_order[0],
        )]
    } else {
        plan.iter()
            .enumerate()
            .map(|(i, (name, path))| (name.clone(), vec![path.clone()], source_order[i]))
            .collect()
    };
    for (name, paths, index) in groups {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let source = &project.sources[index];
        let (format, bext) = crate::export_settings::source_format(source)?;
        let path = output.path(&name);
        crate::wav_delivery::encode_delivery(
            &paths,
            &path,
            settings.encoding.format(format),
            bext.as_ref(),
            source.metadata.grid.frame_count,
            cancelled,
        )?;
        let metadata = crate::SourceMetadata::read_cancellable(&path, cancelled)?;
        if metadata.grid != source.metadata.grid
            || metadata.timing_origin != source.metadata.timing_origin
            || usize::from(metadata.channels) != paths.len()
        {
            return Err(RenderBlocker("Delivery grid or timing changed.".into()));
        }
        fs::OpenOptions::new().write(true).open(&path)?.sync_all()?;
        files.push((
            name,
            crate::project::fingerprint_cancellable(&path, cancelled)?,
            metadata,
        ));
    }
    crate::prepared_analysis::check_cancelled(cancelled)?;
    Ok(PreparedExport {
        output,
        project: project.path().into(),
        identity: project.delivery_identity()?,
        files,
        inputs,
        layout: settings.layout,
    })
}

pub fn export(
    project: &Project,
    destination: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<(), RenderBlocker> {
    let generation = crate::RenderGeneration::default();
    prepare_export(project, destination, cancelled)?.publish(
        project,
        &generation,
        generation.current(),
        cancelled,
    )
}
pub fn validate_export(project: &Project) -> Result<(), RenderBlocker> {
    project.validate_sources()?;
    delivery_plan(project, None, crate::ExportLayout::SeparateMono).map(|_| ())
}

fn delivery_plan(
    project: &Project,
    cancelled: Option<&AtomicBool>,
    layout: crate::ExportLayout,
) -> Result<Vec<(String, std::path::PathBuf)>, RenderBlocker> {
    project.require_alignment()?;
    let RenderState::Current { identity } = project.render() else {
        return Err(RenderBlocker(
            "only a current render can be exported".into(),
        ));
    };
    let report: Report = serde_json::from_slice(&fs::read(identity)?)
        .map_err(|error| RenderBlocker(error.to_string()))?;
    let record = project.render_record().ok_or_else(|| RenderBlocker("A report path is not an accepted supervised render; Analyze and render before exporting.".into()))?;
    {
        if record.report != *identity
            || !record.snapshot.matches(project)
            || crate::project::fingerprint_cancellable(identity, cancelled)? != record.report_sha256
            || record
                .files
                .iter()
                .map(|file| &file.name)
                .ne(report.output.stems.iter())
        {
            return Err(RenderBlocker(
                "Accepted render provenance changed; render again before export.".into(),
            ));
        }
        let directory = identity.parent().unwrap_or(Path::new("."));
        for file in &record.files {
            let path = directory.join(&file.name);
            if crate::project::fingerprint_cancellable(&path, cancelled)? != file.sha256
                || crate::SourceMetadata::read_cancellable(&path, cancelled)? != file.metadata
            {
                return Err(RenderBlocker(
                    "Accepted rendered audio changed; final export is blocked.".into(),
                ));
            }
        }
    }
    if report.output.stems.len() != project.sources.len() {
        return Err(RenderBlocker(
            "render report does not match the project".into(),
        ));
    }
    let names = project
        .sources
        .iter()
        .map(|source| {
            source
                .path
                .file_stem()
                .and_then(|name| name.to_str())
                .map(|stem| {
                    source.channel.map_or_else(
                        || format!("{stem}-clean.wav"),
                        |channel| format!("{stem}-ch{}-clean.wav", channel + 1),
                    )
                })
                .ok_or_else(|| RenderBlocker("source has no usable file name".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if layout == crate::ExportLayout::SeparateMono
        && names
            .iter()
            .map(|name| name.to_lowercase())
            .collect::<HashSet<_>>()
            .len()
            != names.len()
    {
        return Err(RenderBlocker("source names collide at export".into()));
    }
    let rendered = report
        .output
        .stems
        .into_iter()
        .map(|rendered| {
            let rendered = Path::new(&rendered);
            if rendered.file_name() != Some(rendered.as_os_str())
                || !rendered
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
            {
                return Err(RenderBlocker(
                    "render report contains a non-WAV or unsafe path".into(),
                ));
            }
            Ok(identity.parent().unwrap_or(Path::new(".")).join(rendered))
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate(&rendered)?;
    Ok(names.into_iter().zip(rendered).collect())
}
