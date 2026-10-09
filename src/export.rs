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
    files: Vec<(String, std::path::PathBuf, String, crate::SourceMetadata)>,
}

pub fn prepare_export(
    project: &Project,
    destination: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<PreparedExport, RenderBlocker> {
    project.validate_sources_cancellable(cancelled)?;
    let plan = delivery_plan(project, cancelled)?;
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(RenderBlocker("export destination is not empty".into()));
    }
    let output = AtomicOutput::new(destination)?;
    let mut files = Vec::new();
    for (name, rendered) in plan {
        crate::prepared_analysis::check_cancelled(cancelled)?;
        let hash = crate::project::fingerprint_cancellable(&rendered, cancelled)?;
        let metadata = crate::SourceMetadata::read_cancellable(&rendered, cancelled)?;
        {
            use std::io::{Read, Write};
            let mut input = fs::File::open(&rendered)?;
            let mut destination = fs::File::create(output.path(&name))?;
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
        files.push((name, rendered, hash, metadata));
    }
    crate::prepared_analysis::check_cancelled(cancelled)?;
    Ok(PreparedExport {
        output,
        project: project.path().into(),
        identity: project.delivery_identity()?,
        files,
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
        delivery_plan(project, cancelled)?;
        for (name, rendered, hash, metadata) in &self.files {
            if crate::project::fingerprint_cancellable(rendered, cancelled)? != *hash
                || crate::project::fingerprint_cancellable(&self.output.path(name), cancelled)?
                    != *hash
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
    delivery_plan(project, None).map(|_| ())
}

fn delivery_plan(
    project: &Project,
    cancelled: Option<&AtomicBool>,
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
                .map(|stem| format!("{stem}-clean.wav"))
                .ok_or_else(|| RenderBlocker("source has no usable file name".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if names.iter().collect::<HashSet<_>>().len() != names.len() {
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
