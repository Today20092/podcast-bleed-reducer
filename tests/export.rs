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

#[test]
fn exports_current_render_with_delivery_names() {
    let dir = temp_dir();
    let source = dir.join("host.wav");
    wav(&source);
    let original_bytes = fs::read(&source).unwrap();
    let render = dir.join("render");
    fs::create_dir(&render).unwrap();
    wav(&render.join("01-host.wav"));
    fs::write(
        render.join("report.json"),
        r#"{"output":{"stems":["01-host.wav"]}}"#,
    )
    .unwrap();
    let mut project =
        Project::create(dir.join("episode.cleanslate"), vec![source.clone()]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .mark_render_current(render.join("report.json"))
        .unwrap();

    let destination = dir.join("delivery");
    export(&project, &destination, None).unwrap();

    assert!(destination.join("host-clean.wav").is_file());
    assert!(!destination.join("report.json").exists());
    assert_eq!(fs::read(source).unwrap(), original_bytes);
    fs::remove_dir_all(dir).unwrap();
}

fn current_project(dir: &Path) -> Project {
    fs::create_dir_all(dir).unwrap();
    let source = dir.join("host.wav");
    wav(&source);
    let render = dir.join("render");
    fs::create_dir(&render).unwrap();
    wav(&render.join("01-host.wav"));
    fs::write(
        render.join("report.json"),
        r#"{"output":{"stems":["01-host.wav"]}}"#,
    )
    .unwrap();
    let mut project = Project::create(dir.join("episode.cleanslate"), vec![source]).unwrap();
    project.set_supplied_alignment(true).unwrap();
    project
        .mark_render_current(render.join("report.json"))
        .unwrap();
    project
}

#[test]
fn accepts_empty_folder_but_rejects_collisions_without_partial_output() {
    let dir = temp_dir();
    let project = current_project(&dir);
    let empty = dir.join("empty");
    fs::create_dir(&empty).unwrap();
    export(&project, &empty, None).unwrap();
    assert!(empty.join("host-clean.wav").is_file());

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
