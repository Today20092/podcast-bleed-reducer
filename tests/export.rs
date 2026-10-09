use bwavfile::{WaveFmt, WaveWriter};
use rust_app::{Project, export};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-export-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn wav(path: &Path) {
    let writer = WaveWriter::create(path, WaveFmt::new_pcm_mono(48_000, 16)).unwrap();
    let mut frames = writer.audio_frame_writer().unwrap();
    frames.write_frames(&[0.0_f32, 0.25]).unwrap();
    frames.end().unwrap();
}

#[path = "common/supervised_fixture.rs"]
mod fixture;

#[test]
fn exports_current_render_with_delivery_names() {
    let dir = temp_dir();
    let project = current_project(&dir);
    let source = project.sources[0].path.clone();
    let original = fs::read(&source).unwrap();
    export(&project, &dir.join("delivery"), None).unwrap();
    assert!(dir.join("delivery/speaker-0-clean.wav").is_file());
    assert!(dir.join("delivery/speaker-1-clean.wav").is_file());
    assert!(!dir.join("delivery/report.json").exists());
    assert_eq!(fs::read(source).unwrap(), original);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn staged_delivery_rechecks_cancellation_generation_saved_edits_and_copied_bytes() {
    let dir = temp_dir();
    let project = current_project(&dir);
    let gate = rust_app::RenderGeneration::default();
    let prior = dir.join("prior");
    export(&project, &prior, None).unwrap();
    let accepted = fs::read(prior.join("speaker-0-clean.wav")).unwrap();
    for kind in 0..4 {
        let destination = dir.join(format!("obsolete-{kind}"));
        let expected = gate.current();
        let prepared = rust_app::prepare_export(&project, &destination, None).unwrap();
        let cancel = AtomicBool::new(kind == 0);
        match kind {
            1 => {
                gate.invalidate();
            }
            2 => {
                let mut edited = Project::open(project.path()).unwrap();
                let id = edited.reference_labels()[0].id;
                edited.remove_reference_label(id).unwrap();
            }
            3 => {
                let staging = fs::read_dir(&dir)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .find(|path| {
                        path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(".obsolete-3.")
                    })
                    .unwrap();
                fs::write(
                    staging.join("speaker-0-clean.wav"),
                    b"changed during copy acceptance",
                )
                .unwrap();
            }
            _ => {}
        }
        assert!(
            prepared
                .publish(&project, &gate, expected, Some(&cancel))
                .is_err()
        );
        assert!(!destination.exists());
        assert_eq!(
            fs::read(prior.join("speaker-0-clean.wav")).unwrap(),
            accepted
        );
        if kind == 2 {
            project.save().unwrap();
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
fn current_project(dir: &Path) -> Project {
    fs::create_dir_all(dir).unwrap();
    let (fixture_root, old_project, audio, evidence) = fixture::session();
    let paths = old_project
        .sources
        .iter()
        .map(|source| {
            let path = dir.join(source.path.file_name().unwrap());
            fs::copy(&source.path, &path).unwrap();
            path
        })
        .collect();
    let mut project = Project::create(dir.join("episode.cleanslate"), paths).unwrap();
    project.set_supplied_alignment(true).unwrap();
    for label in old_project.reference_labels() {
        project
            .add_reference_label(label.range, label.active_speakers.clone(), &[])
            .unwrap();
    }
    let progress = std::sync::atomic::AtomicU8::new(0);
    let cancel = AtomicBool::new(false);
    let analysis = rust_app::EditorAnalysisWorker::default()
        .analyze_verified_evidence(&project, &audio, &evidence, &progress, &cancel)
        .unwrap();
    let effective = analysis
        .install(
            &mut project,
            &mut rust_app::EffectiveEnvelopeCache::default(),
            Some(&evidence.runtime),
            None,
        )
        .unwrap();
    let snapshot = rust_app::SupervisedRenderSnapshot::capture(
        &project,
        &analysis,
        &effective,
        &evidence.runtime,
    )
    .unwrap();
    let generation = rust_app::RenderGeneration::default();
    rust_app::prepare_supervised_render(
        &project,
        &analysis,
        &effective,
        &snapshot,
        &dir.join("render"),
        &progress,
        None,
    )
    .unwrap()
    .publish(
        &project,
        &evidence.runtime,
        &generation,
        generation.current(),
        None,
    )
    .unwrap()
    .accept(&mut project)
    .unwrap();
    fs::remove_dir_all(fixture_root).unwrap();
    project
}
#[test]
fn a_legacy_report_path_cannot_establish_current_supervised_delivery() {
    let dir = temp_dir();
    let source = dir.join("host.wav");
    wav(&source);
    let output = dir.join("render");
    fs::create_dir(&output).unwrap();
    wav(&output.join("01-host.wav"));
    fs::write(
        output.join("report.json"),
        r#"{"output":{"stems":["01-host.wav"]}}"#,
    )
    .unwrap();
    let mut project = Project::create(dir.join("episode.cleanslate"), vec![source]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .mark_render_current(output.join("report.json"))
        .unwrap();
    assert!(
        export(&project, &dir.join("delivery"), None).is_err(),
        "Report paths cannot bypass accepted supervised provenance"
    );
    assert!(!dir.join("delivery").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_export_never_removes_another_attempts_staging_directory() {
    let dir = temp_dir();
    let project = current_project(&dir);
    let destination = dir.join("delivery");
    let existing = dir.join(format!(".delivery.{}.tmp", std::process::id()));
    fs::create_dir(&existing).unwrap();
    fs::write(
        existing.join("unrelated-accepted-by-another-worker"),
        b"retain",
    )
    .unwrap();
    export(&project, &destination, None).unwrap();
    assert_eq!(
        fs::read(existing.join("unrelated-accepted-by-another-worker")).unwrap(),
        b"retain"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn accepts_empty_folder_but_rejects_collisions_without_partial_output() {
    let dir = temp_dir();
    let project = current_project(&dir);
    let empty = dir.join("empty");
    fs::create_dir(&empty).unwrap();
    export(&project, &empty, None).unwrap();
    assert!(empty.join("speaker-0-clean.wav").is_file());

    let collision = dir.join("collision");
    fs::create_dir(&collision).unwrap();
    fs::write(collision.join("host-clean.wav"), b"keep").unwrap();
    assert!(export(&project, &collision, None).is_err());
    assert_eq!(fs::read(collision.join("host-clean.wav")).unwrap(), b"keep");
    assert!(!collision.join("report.json").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn blocks_stale_render_and_cancellation_without_partial_output() {
    let dir = temp_dir();
    let mut project = current_project(&dir);
    project.protect_range(0, 1).unwrap();
    assert!(export(&project, &dir.join("stale"), None).is_err());
    assert!(!dir.join("stale").exists());

    let project = current_project(&dir.join("second"));
    let destination = dir.join("cancelled");
    assert!(export(&project, &destination, Some(&AtomicBool::new(true))).is_err());
    assert!(!destination.exists());
    fs::remove_dir_all(dir).unwrap();
}
