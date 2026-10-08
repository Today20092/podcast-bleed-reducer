use crate::{Project, RenderBlocker, RenderState, audio_io::AtomicOutput, validate};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Deserialize)]
struct Report {
    output: Output,
}

#[derive(Deserialize)]
struct Output {
    stems: Vec<String>,
}

pub fn export(
    project: &Project,
    destination: &Path,
    cancelled: Option<&AtomicBool>,
) -> Result<(), RenderBlocker> {
    project.validate_sources()?;
    let plan = delivery_plan(project)?;
    if destination.exists() && fs::read_dir(destination)?.next().is_some() {
        return Err(RenderBlocker("export destination is not empty".into()));
    }
    let output = AtomicOutput::new(destination)?;
    for (name, rendered) in plan {
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err(RenderBlocker("cancelled".into()));
        }
        fs::copy(rendered, output.path(&name))?;
    }
    if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err(RenderBlocker("cancelled".into()));
    }
    project.validate_sources()?;
    output.commit()
}

pub fn validate_export(project: &Project) -> Result<(), RenderBlocker> {
    delivery_plan(project).map(|_| ())
}

fn delivery_plan(project: &Project) -> Result<Vec<(String, std::path::PathBuf)>, RenderBlocker> {
    let RenderState::Current { identity } = project.render() else {
        return Err(RenderBlocker(
            "only a current render can be exported".into(),
        ));
    };
    let report: Report = serde_json::from_slice(&fs::read(identity)?)
        .map_err(|error| RenderBlocker(error.to_string()))?;
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
