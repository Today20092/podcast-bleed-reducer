use rust_app::{Comparison, Project, RenderState, ReviewState, ReviewWorkspace};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-review-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn report_drives_selection_and_comparison_without_losing_context() {
    let dir = temp_dir();
    let report = dir.join("report.json");
    fs::write(&report, r#"{"regions":[{"start":48000,"end":96000,"decision":"Owned","owners":[0],"affected_track":1,"applied_attenuation_db":-18,"evidence_reasons":["speech and energy agree"],"review_warning":"broad"},{"start":48000,"end":96000,"decision":"Owned","owners":[0],"affected_track":2,"applied_attenuation_db":-18,"evidence_reasons":["speech and energy agree"],"review_warning":"broad"}],"output":{"stems":["01-a.wav","02-b.wav"]}}"#).unwrap();

    let mut review = ReviewWorkspace::open(&report, &[]).unwrap();
    assert_eq!(review.warnings.len(), 1);
    review.select_warning(0);
    assert_eq!(review.playhead(), 48_000);
    assert_eq!(review.selection(), Some((48_000, 96_000)));
    assert_eq!(review.highlighted_tracks(), &[1, 2]);
    review.set_comparison(Comparison::Processed);
    assert_eq!(review.playhead(), 48_000);
    assert_eq!(review.selection(), Some((48_000, 96_000)));
    let source = dir.join("voice.wav");
    fs::write(&source, b"source").unwrap();
    let mut project = Project::create(dir.join("episode.cleanslate"), vec![source]).unwrap();
    let warning_key = review.warnings[0].key();
    project
        .set_review_state(&warning_key, ReviewState::Reviewed)
        .unwrap();
    assert_eq!(
        Project::open(project.path())
            .unwrap()
            .review_states()
            .get(&warning_key),
        Some(&ReviewState::Reviewed)
    );
    assert_eq!(
        ReviewWorkspace::open(&report, &[]).unwrap().warnings.len(),
        1
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn protections_combine_and_make_the_render_stale() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"source").unwrap();
    let mut project = Project::create(dir.join("episode.cleanslate"), vec![source]).unwrap();
    let rendered = dir.join("render/report.json");
    fs::create_dir_all(rendered.parent().unwrap()).unwrap();
    fs::write(&rendered, b"{}").unwrap();
    project.mark_render_current(&rendered).unwrap();

    project.protect_track(0).unwrap();
    project.protect_range(10, 20).unwrap();
    project.protect_region(&[1, 2], 30, 40).unwrap();
    assert_eq!(
        project.protections()[2..],
        [
            rust_app::Protection {
                track: Some(1),
                start: 30,
                end: 40
            },
            rust_app::Protection {
                track: Some(2),
                start: 30,
                end: 40
            },
        ]
    );
    assert_eq!(project.render(), &RenderState::Stale);
    let reopened = Project::open(project.path()).unwrap();
    assert_eq!(reopened.protections().len(), 4);
    assert_eq!(reopened.review_identity(), Some(rendered.as_path()));
    fs::remove_dir_all(dir).unwrap();
}
