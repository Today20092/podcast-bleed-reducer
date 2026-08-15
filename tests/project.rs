use rust_app::{AutomixParameters, Project, RenderState, ReviewState, WorkflowStep};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cleanslate-project-{}-{}",
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
fn project_reopens_with_workflow_and_review_state() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"source audio identity").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let render_report = dir.join("render-1/report.json");
    fs::create_dir(dir.join("render-1")).unwrap();
    fs::write(&render_report, b"{}").unwrap();

    let mut project = Project::create(&project_path, vec![source]).unwrap();
    project
        .replace_protections(vec![rust_app::Protection {
            track: Some(0),
            start: 10,
            end: 20,
        }])
        .unwrap();
    project
        .set_review_state("broad-attenuation", ReviewState::Reviewed)
        .unwrap();
    let parameters = AutomixParameters {
        attenuation_db: -12.0,
        ..AutomixParameters::default()
    };
    project.set_parameters(parameters).unwrap();
    project.mark_render_current(&render_report).unwrap();

    let reopened = Project::open(&project_path).unwrap();
    assert_eq!(reopened.step(), WorkflowStep::Review);
    assert_eq!(reopened.protections().len(), 1);
    assert_eq!(reopened.parameters(), parameters);
    assert_eq!(
        reopened.review_states()["broad-attenuation"],
        ReviewState::Reviewed
    );
    assert_eq!(
        reopened.render(),
        &RenderState::Current {
            identity: render_report
        }
    );
    assert!(reopened.sources[0].available);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn changed_source_requires_relink_and_full_revalidation() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    let unchanged = dir.join("guest.wav");
    fs::write(&source, b"original").unwrap();
    fs::write(&unchanged, b"unchanged").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let mut project = Project::create(&project_path, vec![source.clone(), unchanged]).unwrap();
    project
        .mark_render_current(dir.join("missing-report.json"))
        .unwrap();

    fs::write(&source, b"changed source").unwrap();
    let reopened = Project::open(&project_path).unwrap();

    assert!(!reopened.sources[0].available);
    assert!(reopened.sources[1].available);
    assert_eq!(reopened.step(), WorkflowStep::Import);
    assert_eq!(reopened.render(), &RenderState::Stale);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_committed_render_reopens_stale() {
    let dir = temp_dir();
    let source = dir.join("voice.wav");
    fs::write(&source, b"original").unwrap();
    let project_path = dir.join("episode.cleanslate");
    let mut project = Project::create(&project_path, vec![source]).unwrap();
    project
        .mark_render_current(dir.join("removed-render/report.json"))
        .unwrap();

    let reopened = Project::open(&project_path).unwrap();
    assert_eq!(reopened.step(), WorkflowStep::Process);
    assert_eq!(reopened.render(), &RenderState::Stale);

    fs::remove_dir_all(dir).unwrap();
}
