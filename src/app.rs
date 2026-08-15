use crate::{
    Comparison, Project, RenderState, ReviewState, ReviewWorkspace, RunRequest, WorkflowStep,
    export, run_with_progress, validate,
};
use iced::{
    Element, Fill, Subscription, Task, Theme,
    widget::{button, column, container, row, scrollable, text},
};
use std::{
    path::PathBuf,
    process::{Child, Command},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver},
    },
    time::{SystemTime, UNIX_EPOCH},
};

fn dialog_start_directory() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|path| path.join("Documents"))
        .filter(|path| path.is_dir())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        })
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    #[test]
    fn file_dialog_starts_in_an_existing_local_directory() {
        assert!(super::dialog_start_directory().is_dir());
    }
}

#[derive(Default)]
pub struct CleanSlateApp {
    project: Option<Project>,
    messages: Vec<String>,
    processing: Option<Processing>,
    review: Option<ReviewWorkspace>,
    playback: Vec<Child>,
}

struct Processing {
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU8>,
    result: Receiver<Result<String, String>>,
}

#[derive(Debug, Clone)]
enum Message {
    NewProject,
    OpenProject,
    Relink(usize),
    Validate,
    Process,
    Cancel,
    Tick,
    SelectRegion(usize),
    SelectWarning(usize),
    Original,
    Processed,
    Audition,
    ToggleReviewed,
    ProtectRegion,
    ProtectTrack(usize),
    ProtectSelection,
    Export,
}

pub fn run() -> iced::Result {
    iced::application(
        CleanSlateApp::default,
        CleanSlateApp::update,
        CleanSlateApp::view,
    )
    .title("CleanSlate")
    .theme(CleanSlateApp::theme)
    .subscription(CleanSlateApp::subscription)
    .window_size([900.0, 680.0])
    .run()
}

impl CleanSlateApp {
    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn export_current(&mut self) {
        let Some(destination) = rfd::FileDialog::new()
            .set_directory(
                self.project
                    .as_ref()
                    .and_then(|project| project.path().parent())
                    .unwrap_or_else(|| std::path::Path::new(".")),
            )
            .pick_folder()
        else {
            return;
        };
        match export(self.project.as_ref().unwrap(), &destination, None) {
            Ok(()) => {
                self.messages = vec![format!("Export committed to {}", destination.display())]
            }
            Err(error) => self.messages = vec![format!("Export blocked: {error}")],
        }
    }

    fn create_project(&mut self) {
        let stems = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter("WAV audio", &["wav"])
            .pick_files();
        let Some(stems) = stems else { return };
        let save_directory = stems
            .first()
            .and_then(|stem| stem.parent())
            .unwrap_or_else(|| std::path::Path::new("."));
        let Some(path) = rfd::FileDialog::new()
            .set_directory(save_directory)
            .add_filter("CleanSlate project", &["cleanslate"])
            .set_file_name("project.cleanslate")
            .save_file()
        else {
            return;
        };
        match Project::create(path, stems) {
            Ok(project) => {
                self.project = Some(project);
                self.messages.clear();
            }
            Err(error) => self.messages = vec![error.to_string()],
        }
    }

    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter("CleanSlate project", &["cleanslate"])
            .pick_file()
        else {
            return;
        };
        match Project::open(path) {
            Ok(project) => {
                self.project = Some(project);
                self.load_review();
                self.messages.clear();
            }
            Err(error) => self.messages = vec![error.to_string()],
        }
    }

    fn validate(&mut self) {
        let Some(project) = &mut self.project else {
            return;
        };
        match validate(&project.source_paths()) {
            Ok(validation) => {
                self.messages = validation.warnings;
                if let Err(error) = project.mark_validated() {
                    self.messages.push(error.to_string());
                }
            }
            Err(blocker) => self.messages = vec![format!("Render blocker: {blocker}")],
        }
    }

    fn start_processing(&mut self) {
        let Some(project) = &self.project else { return };
        let stems = project.source_paths();
        let protections = project.protections().to_vec();
        let parent = project
            .path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
        let output_dir = parent.join(format!("render-{id}"));
        let report = output_dir.join("report.json");
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU8::new(0));
        let request_cancelled = Arc::clone(&cancelled);
        let request_progress = Arc::clone(&progress);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let outcome = run_with_progress(
                RunRequest {
                    stems,
                    output_dir,
                    regions: vec![],
                    protections,
                    cancelled: Some(request_cancelled),
                },
                Some(request_progress),
            )
            .map(|_| report.display().to_string())
            .map_err(|error| error.to_string());
            let _ = sender.send(outcome);
        });
        self.messages = vec!["Processing started".into()];
        self.processing = Some(Processing {
            cancelled,
            progress,
            result,
        });
    }

    fn load_review(&mut self) {
        self.review = self.project.as_ref().and_then(|project| {
            let identity = project.review_identity()?;
            ReviewWorkspace::open(identity, &project.source_paths()).ok()
        });
    }

    fn audition(&mut self) {
        for mut child in self.playback.drain(..) {
            let _ = child.kill();
        }
        let (Some(project), Some(review)) = (&self.project, &self.review) else {
            return;
        };
        let Some((start, end)) = review.selection() else {
            return;
        };
        let paths: Vec<_> = match review.comparison() {
            Comparison::Original => project
                .sources
                .iter()
                .map(|source| source.path.clone())
                .collect(),
            Comparison::Processed => project
                .review_identity()
                .map(|report| {
                    review
                        .output_stems
                        .iter()
                        .map(|stem| report.parent().unwrap().join(stem))
                        .collect()
                })
                .unwrap_or_default(),
        };
        let script = "$p=New-Object -ComObject WMPlayer.OCX; $p.URL=$args[0]; $p.controls.currentPosition=[double]$args[1]; $p.controls.play(); Start-Sleep -Milliseconds ([int]$args[2]); $p.controls.stop()";
        self.playback = paths
            .into_iter()
            .filter_map(|path| {
                Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-WindowStyle",
                        "Hidden",
                        "-Command",
                        script,
                        &path.display().to_string(),
                        &(start as f64 / 48_000.0).to_string(),
                        &((end - start) * 1000 / 48_000).to_string(),
                    ])
                    .spawn()
                    .ok()
            })
            .collect();
    }

    fn poll_processing(&mut self) {
        let Some(processing) = &self.processing else {
            return;
        };
        let Ok(result) = processing.result.try_recv() else {
            return;
        };
        self.processing = None;
        match result {
            Ok(identity) => {
                let project = self.project.as_mut().unwrap();
                self.messages = vec!["Render committed; ready for Review".into()];
                if let Err(error) = project.mark_render_current(identity) {
                    self.messages.push(error.to_string());
                }
                self.load_review();
            }
            Err(error) => self.messages = vec![error],
        }
    }
}

impl CleanSlateApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewProject => self.create_project(),
            Message::OpenProject => self.open_project(),
            Message::Relink(index) => {
                let start = self
                    .project
                    .as_ref()
                    .and_then(|project| project.sources.get(index))
                    .and_then(|source| source.path.parent())
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or_else(dialog_start_directory);
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(start)
                    .add_filter("WAV audio", &["wav"])
                    .pick_file()
                    && let Err(error) = self.project.as_mut().unwrap().relink(index, path)
                {
                    self.messages = vec![error.to_string()];
                }
            }
            Message::Validate => self.validate(),
            Message::Process => self.start_processing(),
            Message::Cancel => {
                if let Some(processing) = &self.processing {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
            }
            Message::Tick => self.poll_processing(),
            Message::SelectRegion(index) => {
                if let Some(review) = &mut self.review {
                    review.select(index);
                }
            }
            Message::SelectWarning(index) => {
                if let Some(review) = &mut self.review {
                    review.select(index);
                }
                self.audition();
            }
            Message::Original | Message::Processed => {
                if let Some(review) = &mut self.review {
                    review.set_comparison(if matches!(message, Message::Original) {
                        Comparison::Original
                    } else {
                        Comparison::Processed
                    });
                }
            }
            Message::Audition => self.audition(),
            Message::ToggleReviewed => {
                if let (Some(project), Some(region)) = (
                    self.project.as_mut(),
                    self.review
                        .as_ref()
                        .and_then(ReviewWorkspace::selected_region),
                ) {
                    let key = region.key();
                    let state = if project.review_states().get(&key) == Some(&ReviewState::Reviewed)
                    {
                        ReviewState::Unreviewed
                    } else {
                        ReviewState::Reviewed
                    };
                    if let Err(error) = project.set_review_state(key, state) {
                        self.messages.push(error.to_string());
                    }
                }
            }
            Message::ProtectRegion => {
                if let (Some(project), Some(region)) = (
                    self.project.as_mut(),
                    self.review
                        .as_ref()
                        .and_then(ReviewWorkspace::selected_region),
                ) && let Err(error) = project.protect_range(region.start, region.end)
                {
                    self.messages.push(error.to_string());
                }
                if let Some(review) = &mut self.review {
                    review.set_comparison(Comparison::Original);
                }
            }
            Message::ProtectTrack(track) => {
                if let Some(project) = &mut self.project
                    && let Err(error) = project.protect_track(track)
                {
                    self.messages.push(error.to_string());
                }
                if let Some(review) = &mut self.review {
                    review.set_comparison(Comparison::Original);
                }
            }
            Message::ProtectSelection => {
                if let (Some(project), Some((start, end))) = (
                    self.project.as_mut(),
                    self.review.as_ref().and_then(ReviewWorkspace::selection),
                ) && let Err(error) = project.protect_range(start.min(end), start.max(end))
                {
                    self.messages.push(error.to_string());
                }
            }
            Message::Export => self.export_current(),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.processing.is_some() {
            iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            button("New project").on_press(Message::NewProject),
            button("Open project").on_press(Message::OpenProject),
        ]
        .spacing(10);
        let current = self
            .project
            .as_ref()
            .map_or(WorkflowStep::Import, Project::step);
        let rail = row([
            WorkflowStep::Import,
            WorkflowStep::Validate,
            WorkflowStep::Process,
            WorkflowStep::Review,
            WorkflowStep::Export,
        ]
        .into_iter()
        .map(|step| {
            text(if step == current {
                format!("● {step:?}")
            } else {
                format!("○ {step:?}")
            })
            .into()
        }))
        .spacing(16);

        let mut content = column![].spacing(10);
        if let Some(project) = &self.project {
            for (index, source) in project.sources.iter().enumerate() {
                let mut source_row = row![text(source.path.display().to_string())].spacing(10);
                if !source.available {
                    source_row = source_row.push(button("Relink").on_press(Message::Relink(index)));
                }
                content = content.push(source_row);
            }
        }

        if self.project.is_none() || current == WorkflowStep::Import {
            content = content.push(button("Import stems").on_press(Message::NewProject));
        } else if current == WorkflowStep::Validate {
            content = content.push(button("Validate session").on_press(Message::Validate));
        } else if current == WorkflowStep::Process {
            if let Some(processing) = &self.processing {
                content = content
                    .push(text(format!(
                        "Processing... {}%",
                        processing.progress.load(Ordering::Relaxed)
                    )))
                    .push(button("Cancel processing").on_press(Message::Cancel));
            } else {
                content = content.push(button("Process session").on_press(Message::Process));
            }
        } else if current == WorkflowStep::Review {
            let Some(project) = &self.project else {
                unreachable!()
            };
            let stale = matches!(project.render(), RenderState::Stale);
            content = content.push(text(match project.render() {
                RenderState::Current { .. } => "Render current",
                RenderState::Stale => "Render stale",
                RenderState::None => "No render",
            }));
            if stale && self.processing.is_none() {
                content = content.push(button("Rerender protections").on_press(Message::Process));
            }
            if let Some(review) = &self.review {
                content = content.push(text("Synchronized waveforms").size(22));
                for (track, source) in project.sources.iter().enumerate() {
                    let marker = if review.highlighted_tracks().contains(&track) {
                        "▶"
                    } else {
                        " "
                    };
                    content = content.push(
                        container(text(format!(
                            "{marker} {}",
                            source
                                .path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                        )))
                        .padding(10)
                        .width(Fill)
                        .style(container::rounded_box),
                    );
                }
                content = content.push(
                    row![
                        text(format!(
                            "Playhead {:.2}s",
                            review.playhead() as f32 / 48_000.0
                        )),
                        button("Original").on_press(Message::Original),
                        button("Processed").on_press_maybe((!stale).then_some(Message::Processed)),
                        button("Audition selection").on_press(Message::Audition),
                    ]
                    .spacing(10),
                );
                content = content.push(text("Attenuation regions").size(22));
                for (index, region) in review
                    .regions
                    .iter()
                    .enumerate()
                    .filter(|(_, region)| region.review_warning.is_some())
                {
                    content = content.push(
                        button(region.review_warning.as_deref().unwrap())
                            .on_press(Message::SelectWarning(index)),
                    );
                }
                for (index, region) in review.regions.iter().enumerate() {
                    let title = format!(
                        "{:.2}-{:.2}s | {} dB | {:?} | {:?} | {}",
                        region.start as f32 / 48_000.0,
                        region.end as f32 / 48_000.0,
                        region.applied_attenuation_db,
                        region.decision,
                        region.evidence_reasons,
                        region.review_warning.as_deref().unwrap_or("")
                    );
                    content =
                        content.push(button(text(title)).on_press(Message::SelectRegion(index)));
                }
                if let Some(region) = review.selected_region() {
                    let reviewed =
                        project.review_states().get(&region.key()) == Some(&ReviewState::Reviewed);
                    let mut actions = row![
                        button(if reviewed {
                            "Mark unreviewed"
                        } else {
                            "Mark reviewed"
                        })
                        .on_press(Message::ToggleReviewed),
                        button("Protect this region").on_press(Message::ProtectRegion),
                        button("Protect selected range").on_press(Message::ProtectSelection),
                    ]
                    .spacing(10);
                    if let Some(track) = region.affected_track {
                        actions = actions.push(
                            button(text(format!("Protect track {}", track + 1)))
                                .on_press(Message::ProtectTrack(track)),
                        );
                    }
                    content = content.push(actions);
                }
            }
            if matches!(project.render(), RenderState::Current { .. }) {
                content = content.push(button("Export current render").on_press(Message::Export));
            }
        } else {
            content = content.push(text("Export complete."));
        }
        for message in &self.messages {
            content = content.push(text(message));
        }

        container(scrollable(
            column![toolbar, text("CleanSlate").size(34), rail, content,].spacing(14),
        ))
        .padding(20)
        .width(Fill)
        .height(Fill)
        .into()
    }
}
