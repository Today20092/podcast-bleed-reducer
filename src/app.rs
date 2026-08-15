use crate::{
    AnalysisPreview, Comparison, PreviewParameters, Project, RenderState, ReviewState,
    ReviewWorkspace, RunRequest, WorkflowStep, analyze_preview,
    audition::{AuditionMode, LiveAudition},
    export, run_with_progress,
    timeline::{TimelineEvent, TimelineViewport},
    validate,
    waveform::{self, TimelineCanvas, WaveformCache},
};
use iced::{
    Element, Fill, Length, Subscription, Task, Theme, keyboard,
    widget::{
        Space, button, canvas, column, container, progress_bar, responsive, row, scrollable,
        slider, text,
    },
    window,
};
use std::{
    path::PathBuf,
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

fn available_project_path(directory: &std::path::Path) -> PathBuf {
    let mut path = directory.join("CleanSlate.cleanslate");
    for suffix in 2.. {
        if !path.exists() {
            return path;
        }
        path = directory.join(format!("CleanSlate-{suffix}.cleanslate"));
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn both_import_entry_points_share_deterministic_queueing() {
        let paths = [
            PathBuf::from("B.wav"),
            PathBuf::from("a.wav"),
            PathBuf::from("B.wav"),
        ];
        let mut picker = super::CleanSlateApp::default();
        picker.queue_stems(paths.clone());
        let mut drop = super::CleanSlateApp::default();
        for path in paths {
            drop.queue_stems([path]);
        }
        assert_eq!(picker.pending_stems, drop.pending_stems);
        assert_eq!(
            picker.pending_stems,
            [PathBuf::from("a.wav"), PathBuf::from("B.wav")]
        );
    }

    #[test]
    fn inspector_stacks_only_in_compact_windows() {
        assert!(super::compact_workspace(1280.0));
        assert!(!super::compact_workspace(1600.0));
    }

    #[test]
    fn file_dialog_starts_in_an_existing_local_directory() {
        assert!(super::dialog_start_directory().is_dir());
    }

    #[test]
    fn dropped_stems_never_overwrite_an_existing_project() {
        let directory =
            std::env::temp_dir().join(format!("cleanslate-path-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("CleanSlate.cleanslate"), b"existing").unwrap();
        assert_eq!(
            super::available_project_path(&directory),
            directory.join("CleanSlate-2.cleanslate")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}

pub struct CleanSlateApp {
    project: Option<Project>,
    pending_stems: Vec<PathBuf>,
    messages: Vec<String>,
    processing: Option<Processing>,
    review: Option<ReviewWorkspace>,
    audition: Option<LiveAudition>,
    audition_mode: AuditionMode,
    monitor_db: f32,
    waveforms: Vec<WaveformCache>,
    timeline: TimelineViewport,
    preview_parameters: PreviewParameters,
    preview: Option<AnalysisPreview>,
    preview_processing: Option<PreviewProcessing>,
    preview_error: Option<String>,
}

impl Default for CleanSlateApp {
    fn default() -> Self {
        Self {
            project: None,
            pending_stems: vec![],
            messages: vec![],
            processing: None,
            review: None,
            audition: None,
            audition_mode: AuditionMode::Original,
            monitor_db: -6.0,
            waveforms: vec![],
            timeline: TimelineViewport::default(),
            preview_parameters: PreviewParameters::default(),
            preview: None,
            preview_processing: None,
            preview_error: None,
        }
    }
}

struct Processing {
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU8>,
    result: Receiver<Result<String, String>>,
}

struct PreviewProcessing {
    progress: Arc<AtomicU8>,
    result: Receiver<Result<AnalysisPreview, String>>,
}

#[derive(Debug, Clone)]
enum Message {
    ImportStems,
    OpenProject,
    FileDropped(PathBuf),
    ClearDropped,
    CreateDroppedProject,
    Relink(usize),
    Validate,
    Process,
    Cancel,
    Tick,
    SpeechSensitivity(f32),
    BleedDiscrimination(f32),
    Attenuation(f32),
    SpeechPadding(f32),
    Transition(f32),
    RefreshPreview,
    SelectRegion(usize),
    SelectWarning(usize),
    Original,
    Processed,
    Removed,
    Audition,
    AuditionSelection,
    Stop,
    Previous,
    MonitorGain(f32),
    KeyPressed(keyboard::Key),
    ToggleReviewed,
    ProtectRegion,
    ProtectTrack(usize),
    ProtectSelection,
    Export,
    Timeline(TimelineEvent),
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
    .window(window::Settings {
        maximized: true,
        min_size: Some(iced::Size::new(960.0, 640.0)),
        ..window::Settings::default()
    })
    .run()
}

fn compact_workspace(width: f32) -> bool {
    width < 1400.0
}

impl CleanSlateApp {
    fn install_waveforms(&mut self, waveforms: Vec<WaveformCache>) {
        if let Some(audition) = self.audition.take() {
            audition.stop();
        }
        self.preview = None;
        self.preview_processing = None;
        if let Some(cache) = waveforms.first() {
            self.timeline.set_total_samples(cache.sample_count);
        }
        self.waveforms = waveforms;
    }

    fn queue_stems(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        self.pending_stems.extend(paths);
        self.pending_stems
            .sort_by_key(|path| path.to_string_lossy().to_lowercase());
        self.pending_stems.dedup();
        self.messages.clear();
    }

    fn import_stems(&mut self) {
        let Some(paths) = rfd::FileDialog::new()
            .set_directory(dialog_start_directory())
            .add_filter("WAV audio", &["wav"])
            .pick_files()
        else {
            return;
        };
        self.queue_stems(paths);
    }

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

    fn create_dropped_project(&mut self) {
        let Some(directory) = self.pending_stems.first().and_then(|stem| stem.parent()) else {
            return;
        };
        let project_path = available_project_path(directory);
        let paths = self.pending_stems.clone();
        let validation = match validate(&paths) {
            Ok(validation) => validation,
            Err(blocker) => {
                self.pending_stems.clear();
                self.messages = vec![format!("Render blocker: {blocker}")];
                return;
            }
        };
        match Project::create(&project_path, paths.clone()) {
            Ok(mut project) => {
                self.install_waveforms(
                    waveform::load_or_build(&project_path, &paths).unwrap_or_default(),
                );
                if let Err(error) = project.mark_validated() {
                    self.messages = vec![error.to_string()];
                    return;
                }
                self.project = Some(project);
                self.pending_stems.clear();
                self.messages = validation.warnings;
                self.start_preview();
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
                self.install_waveforms(
                    waveform::load_or_build(project.path(), &project.source_paths())
                        .unwrap_or_default(),
                );
                self.project = Some(project);
                self.load_review();
                self.messages.clear();
                self.start_preview();
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

    fn start_preview(&mut self) {
        let Some(project) = &self.project else { return };
        let stems = project.source_paths();
        if stems.iter().any(|path| !path.is_file()) {
            self.preview_processing = None;
            self.preview_error = Some("source stem is missing; relink it to analyze".into());
            return;
        }
        let protections = project.protections().to_vec();
        let parameters = self.preview_parameters;
        if self.preview.as_ref().is_some_and(|preview| {
            preview.stems == stems
                && preview.parameters == parameters
                && preview.protections == protections
        }) {
            self.preview_processing = None;
            self.preview_error = None;
            return;
        }
        let progress = Arc::new(AtomicU8::new(0));
        let worker_progress = Arc::clone(&progress);
        let (sender, result) = mpsc::channel();
        std::thread::spawn(move || {
            let result = analyze_preview(&stems, &protections, parameters, worker_progress)
                .map_err(|error| error.to_string());
            let _ = sender.send(result);
        });
        self.preview_processing = Some(PreviewProcessing { progress, result });
        self.preview_error = None;
    }

    fn load_review(&mut self) {
        self.review = self.project.as_ref().and_then(|project| {
            let identity = project.review_identity()?;
            ReviewWorkspace::open(identity, &project.source_paths()).ok()
        });
        if let Some(review) = &self.review {
            self.timeline.seek(review.playhead());
            self.timeline.set_selection(review.selection());
        }
    }

    fn audition(&mut self, selection_only: bool) {
        let (Some(project), Some(preview)) = (&self.project, &self.preview) else {
            return;
        };
        let selection = if selection_only {
            self.timeline.selection()
        } else {
            None
        };
        let start = selection.map_or(self.timeline.playhead(), |range| range.0.min(range.1));
        let end = selection.map_or_else(
            || {
                preview
                    .envelopes
                    .iter()
                    .map(|envelope| envelope.sample_count())
                    .max()
                    .unwrap_or(start)
            },
            |range| range.0.max(range.1),
        );
        self.audition.take().inspect(|playing| playing.stop());
        match LiveAudition::start(
            project.source_paths(),
            preview.envelopes.clone(),
            self.audition_mode,
            10.0_f32.powf(self.monitor_db / 20.0),
            start,
            end,
        ) {
            Ok(audition) => self.audition = Some(audition),
            Err(error) => self.messages = vec![error.to_string()],
        }
    }

    fn poll_processing(&mut self) {
        if let Some(processing) = &self.preview_processing
            && let Ok(result) = processing.result.try_recv()
        {
            self.preview_processing = None;
            match result {
                Ok(preview) => {
                    if let Some(audition) = &self.audition {
                        audition.replace_envelopes(preview.envelopes.clone());
                    }
                    self.preview = Some(preview);
                }
                Err(error) => self.preview_error = Some(error),
            }
        }
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

    fn poll_audition(&mut self) {
        let Some(audition) = &self.audition else {
            return;
        };
        self.timeline.seek(audition.position());
        if let Some(review) = &mut self.review {
            review.seek(self.timeline.playhead());
        }
        if audition.is_finished() {
            self.audition = None;
        }
    }

    fn set_audition_mode(&mut self, mode: AuditionMode) {
        self.audition_mode = mode;
        if let Some(audition) = &self.audition {
            audition.set_mode(mode);
        }
    }

    fn seek_to_selection_start(&mut self) {
        let start = self
            .timeline
            .selection()
            .map_or(0, |range| range.0.min(range.1));
        self.timeline.seek(start);
        if let Some(audition) = &self.audition {
            audition.seek(start);
        }
        if let Some(review) = &mut self.review {
            review.seek(start);
        }
    }
}

impl CleanSlateApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImportStems => self.import_stems(),
            Message::OpenProject => self.open_project(),
            Message::FileDropped(path) => {
                if path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
                {
                    self.queue_stems([path]);
                } else {
                    self.messages = vec![format!("Ignored non-WAV file: {}", path.display())];
                }
            }
            Message::ClearDropped => self.pending_stems.clear(),
            Message::CreateDroppedProject => self.create_dropped_project(),
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
                {
                    let project = self.project.as_mut().unwrap();
                    match project.relink(index, path) {
                        Ok(()) => {
                            let waveforms =
                                waveform::load_or_build(project.path(), &project.source_paths())
                                    .unwrap_or_default();
                            self.install_waveforms(waveforms);
                        }
                        Err(error) => self.messages = vec![error.to_string()],
                    }
                }
            }
            Message::Validate => self.validate(),
            Message::Process => self.start_processing(),
            Message::Cancel => {
                if let Some(processing) = &self.processing {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
            }
            Message::Tick => {
                self.poll_processing();
                self.poll_audition();
            }
            Message::SpeechSensitivity(value) => self.preview_parameters.speech_sensitivity = value,
            Message::BleedDiscrimination(value) => {
                self.preview_parameters.bleed_discrimination_db = value
            }
            Message::Attenuation(value) => self.preview_parameters.attenuation_db = value,
            Message::SpeechPadding(value) => self.preview_parameters.speech_padding_ms = value,
            Message::Transition(value) => self.preview_parameters.transition_ms = value,
            Message::RefreshPreview => {
                if let Some(project) = &mut self.project
                    && let Err(error) = project.mark_render_stale()
                {
                    self.messages.push(error.to_string());
                }
                self.start_preview();
            }
            Message::SelectRegion(index) => {
                if let Some(review) = &mut self.review {
                    review.select(index);
                    self.timeline.seek(review.playhead());
                    self.timeline.set_selection(review.selection());
                }
            }
            Message::SelectWarning(index) => {
                if let Some(review) = &mut self.review {
                    review.select(index);
                    self.timeline.seek(review.playhead());
                    self.timeline.set_selection(review.selection());
                }
                self.audition(true);
            }
            Message::Original | Message::Processed | Message::Removed => {
                let mode = match message {
                    Message::Original => AuditionMode::Original,
                    Message::Processed => AuditionMode::Processed,
                    _ => AuditionMode::Removed,
                };
                self.set_audition_mode(mode);
                if mode != AuditionMode::Removed
                    && let Some(review) = &mut self.review
                {
                    review.set_comparison(if matches!(message, Message::Original) {
                        Comparison::Original
                    } else {
                        Comparison::Processed
                    });
                }
            }
            Message::Audition => self.audition(false),
            Message::AuditionSelection => self.audition(true),
            Message::Stop => {
                if let Some(audition) = self.audition.take() {
                    audition.stop();
                }
            }
            Message::Previous => self.seek_to_selection_start(),
            Message::MonitorGain(value) => {
                self.monitor_db = value;
                if let Some(audition) = &self.audition {
                    audition.set_monitor(10.0_f32.powf(value / 20.0));
                }
            }
            Message::KeyPressed(key) => match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Space) => {
                    if let Some(audition) = self.audition.take() {
                        audition.stop();
                    } else {
                        self.audition(false);
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Home) => self.seek_to_selection_start(),
                keyboard::Key::Character("1") => self.set_audition_mode(AuditionMode::Original),
                keyboard::Key::Character("2") => self.set_audition_mode(AuditionMode::Processed),
                keyboard::Key::Character("3") => self.set_audition_mode(AuditionMode::Removed),
                _ => {}
            },
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
                self.start_preview();
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
                self.start_preview();
            }
            Message::ProtectSelection => {
                if let (Some(project), Some((start, end))) = (
                    self.project.as_mut(),
                    self.review.as_ref().and_then(ReviewWorkspace::selection),
                ) && let Err(error) = project.protect_range(start.min(end), start.max(end))
                {
                    self.messages.push(error.to_string());
                }
                self.start_preview();
            }
            Message::Export => self.export_current(),
            Message::Timeline(event) => match event {
                TimelineEvent::Seek(sample)
                | TimelineEvent::Drag(sample)
                | TimelineEvent::DragEnd(sample) => {
                    self.timeline.seek(sample);
                    if let Some(audition) = &self.audition {
                        audition.seek(sample);
                    }
                    if let Some(review) = &mut self.review {
                        review.seek(sample);
                    }
                }
                TimelineEvent::Zoom {
                    cursor_x,
                    width,
                    direction,
                } => self.timeline.zoom_at(cursor_x, width, direction),
            },
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let drops = iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            _ => None,
        });
        let processing = if self.processing.is_some()
            || self.preview_processing.is_some()
            || self.audition.is_some()
        {
            iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };
        Subscription::batch([drops, processing])
    }

    fn view(&self) -> Element<'_, Message> {
        responsive(|size| self.workspace(compact_workspace(size.width))).into()
    }

    fn workspace(&self, compact: bool) -> Element<'_, Message> {
        let current = self
            .project
            .as_ref()
            .map_or(WorkflowStep::Import, Project::step);
        let header = row![
            column![
                text("CLEANSLATE").size(24),
                text("UNTITLED SESSION").size(12)
            ]
            .spacing(2),
            Space::new().width(Fill),
            text(match current {
                WorkflowStep::Import => "READY TO IMPORT",
                WorkflowStep::Validate => "READY TO VALIDATE",
                WorkflowStep::Process => "READY TO ANALYZE",
                WorkflowStep::Review => "ANALYSIS COMPLETE",
                WorkflowStep::Export => "EXPORT COMPLETE",
            })
            .size(12),
            button("Import stems").on_press(Message::ImportStems),
            button("Open session").on_press(Message::OpenProject),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let mut timeline_content = column![].spacing(10).height(Fill);
        if self.project.is_none() && self.pending_stems.is_empty() {
            timeline_content = timeline_content.push(
                container(
                    column![
                        text("NO STEMS IMPORTED").size(22),
                        text("Import synchronized mono WAV stems or drop them anywhere in this window."),
                        button("Import WAV stems").on_press(Message::ImportStems),
                    ]
                    .spacing(12)
                    .align_x(iced::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .height(Fill),
            );
        } else if self.project.is_none() {
            timeline_content = timeline_content.push(text("STEMS READY TO IMPORT").size(18));
            for stem in &self.pending_stems {
                timeline_content = timeline_content.push(text(stem.display().to_string()));
            }
            timeline_content = timeline_content.push(
                row![
                    button(text(format!(
                        "Create session from {} stem(s)",
                        self.pending_stems.len()
                    )))
                    .on_press(Message::CreateDroppedProject),
                    button("Clear selection").on_press(Message::ClearDropped),
                ]
                .spacing(8),
            );
        } else if let Some(project) = &self.project {
            timeline_content = timeline_content.push(
                container(
                    canvas(TimelineCanvas::ruler(&self.timeline, Message::Timeline))
                        .width(Fill)
                        .height(28),
                )
                .padding([0, 12]),
            );
            for (index, source) in project.sources.iter().enumerate() {
                let mut header = row![text(source.path.display().to_string())];
                if !source.available {
                    header = header.push(button("Relink source").on_press(Message::Relink(index)));
                }
                let waveform: Element<'_, Message> = self.waveforms.get(index).map_or_else(
                    || {
                        container(text("Waveform loading…"))
                            .height(80)
                            .center_y(Fill)
                            .into()
                    },
                    |cache| {
                        canvas(TimelineCanvas::waveform(
                            cache,
                            &self.timeline,
                            Message::Timeline,
                            self.preview.as_ref().map(|preview| (preview, index)),
                        ))
                        .width(Fill)
                        .height(80)
                        .into()
                    },
                );
                timeline_content = timeline_content.push(
                    container(column![header, waveform].spacing(8))
                        .padding(12)
                        .width(Fill)
                        .style(container::bordered_box),
                );
            }
            match current {
                WorkflowStep::Import => {}
                WorkflowStep::Validate => {
                    timeline_content = timeline_content
                        .push(button("Validate session").on_press(Message::Validate));
                }
                WorkflowStep::Process => {
                    timeline_content = if let Some(processing) = &self.processing {
                        timeline_content
                            .push(text(format!(
                                "Analyzing session: {}%",
                                processing.progress.load(Ordering::Relaxed)
                            )))
                            .push(button("Cancel analysis").on_press(Message::Cancel))
                    } else {
                        timeline_content.push(button("Analyze session").on_press(Message::Process))
                    };
                }
                WorkflowStep::Review => {
                    let stale = matches!(project.render(), RenderState::Stale);
                    if stale && self.processing.is_none() {
                        timeline_content = timeline_content
                            .push(button("Rerender protections").on_press(Message::Process));
                    }
                    if let Some(review) = &self.review {
                        timeline_content = timeline_content.push(
                            row![
                                button("Original").on_press(Message::Original),
                                button("Processed").on_press(Message::Processed),
                                button("Removed").on_press(Message::Removed),
                                button("Audition selection").on_press(Message::AuditionSelection),
                            ]
                            .spacing(8),
                        );
                        for (index, region) in review.regions.iter().enumerate() {
                            let label = region.review_warning.as_deref().unwrap_or("Review region");
                            let message = if region.review_warning.is_some() {
                                Message::SelectWarning(index)
                            } else {
                                Message::SelectRegion(index)
                            };
                            timeline_content =
                                timeline_content.push(button(label).on_press(message));
                        }
                        if let Some(region) = review.selected_region() {
                            let reviewed = project.review_states().get(&region.key())
                                == Some(&ReviewState::Reviewed);
                            let mut actions = row![
                                button(if reviewed {
                                    "Mark unreviewed"
                                } else {
                                    "Mark reviewed"
                                })
                                .on_press(Message::ToggleReviewed),
                                button("Protect this region").on_press(Message::ProtectRegion),
                                button("Protect selected range")
                                    .on_press(Message::ProtectSelection),
                            ]
                            .spacing(8);
                            if let Some(track) = region.affected_track {
                                actions = actions.push(
                                    button(text(format!("Protect track {}", track + 1)))
                                        .on_press(Message::ProtectTrack(track)),
                                );
                            }
                            timeline_content = timeline_content.push(actions);
                        }
                    }
                    if matches!(project.render(), RenderState::Current { .. }) {
                        timeline_content = timeline_content
                            .push(button("Export current render").on_press(Message::Export));
                    }
                }
                WorkflowStep::Export => {
                    timeline_content = timeline_content.push(text("Export complete."));
                }
            }
        }
        for message in &self.messages {
            timeline_content = timeline_content.push(text(message));
        }

        let timeline = column![
            text("SESSION TIMELINE").size(12),
            scrollable(timeline_content).height(Fill),
        ]
        .spacing(10)
        .width(Fill)
        .height(Fill);
        let preview_state: Element<'_, Message> = if let Some(processing) = &self.preview_processing
        {
            let progress = processing.progress.load(Ordering::Relaxed) as f32;
            column![
                text(format!("Recalculating preview: {progress:.0}%")),
                progress_bar(0.0..=100.0, progress),
                text(if self.preview.is_some() {
                    "Showing previous completed preview"
                } else {
                    "No completed preview yet"
                }),
            ]
            .spacing(6)
            .into()
        } else if let Some(error) = &self.preview_error {
            text(format!("Preview failed: {error}")).into()
        } else if let Some(preview) = &self.preview {
            text(format!(
                "Preview ready · {} ownership regions",
                preview.regions.len()
            ))
            .into()
        } else {
            text("Preview not analyzed").into()
        };
        let parameters = &self.preview_parameters;
        let inspector = container(
            column![
                text("ANALYSIS PREVIEW").size(12),
                text("Session parameters").size(20),
                text("Live preview only — final render is unchanged."),
                preview_state,
                text(format!(
                    "Speech sensitivity: {:.2}",
                    parameters.speech_sensitivity
                )),
                slider(
                    0.1..=0.9,
                    parameters.speech_sensitivity,
                    Message::SpeechSensitivity
                )
                .step(0.01)
                .on_release(Message::RefreshPreview),
                text(format!(
                    "Bleed discrimination: {:.1} dB",
                    parameters.bleed_discrimination_db
                )),
                slider(
                    0.0..=18.0,
                    parameters.bleed_discrimination_db,
                    Message::BleedDiscrimination
                )
                .step(0.5)
                .on_release(Message::RefreshPreview),
                text(format!("Attenuation: {:.1} dB", parameters.attenuation_db)),
                slider(
                    -30.0..=-3.0,
                    parameters.attenuation_db,
                    Message::Attenuation
                )
                .step(1.0)
                .on_release(Message::RefreshPreview),
                text(format!(
                    "Speech padding: {:.0} ms",
                    parameters.speech_padding_ms
                )),
                slider(
                    0.0..=1000.0,
                    parameters.speech_padding_ms,
                    Message::SpeechPadding
                )
                .step(16.0)
                .on_release(Message::RefreshPreview),
                text(format!(
                    "Transition time: {:.0} ms",
                    parameters.transition_ms
                )),
                slider(1.0..=250.0, parameters.transition_ms, Message::Transition)
                    .step(1.0)
                    .on_release(Message::RefreshPreview),
                text(
                    "Overlay: preserved solid · attenuated hatch · uncertain amber · protected blue"
                ),
            ]
            .spacing(12),
        )
        .padding(18)
        .width(if compact { Fill } else { Length::Fixed(280.0) })
        .style(container::bordered_box);
        let transport = row![
            text(format!(
                "{:02}:{:06.3}",
                self.timeline.playhead() / 48_000 / 60,
                (self.timeline.playhead() % (48_000 * 60)) as f64 / 48_000.0
            )),
            Space::new().width(Fill),
            button("Previous").on_press(Message::Previous),
            button(if self.audition.is_some() {
                "Playing"
            } else {
                "Play"
            })
            .on_press_maybe(self.preview.is_some().then_some(Message::Audition)),
            button("Stop").on_press_maybe(self.audition.is_some().then_some(Message::Stop)),
            Space::new().width(Fill),
            text("AUDITION"),
            button(if self.audition_mode == AuditionMode::Original {
                "Original ✓"
            } else {
                "Original"
            })
            .on_press(Message::Original),
            button(if self.audition_mode == AuditionMode::Processed {
                "Processed ✓"
            } else {
                "Processed"
            })
            .on_press(Message::Processed),
            button(if self.audition_mode == AuditionMode::Removed {
                "Removed ✓"
            } else {
                "Removed"
            })
            .on_press(Message::Removed),
            text(format!("Monitor {:+.1} dB", self.monitor_db)),
            slider(-24.0..=6.0, self.monitor_db, Message::MonitorGain)
                .step(1.0)
                .width(120),
        ]
        .spacing(8)
        .align_y(iced::Center);
        let editor: Element<'_, Message> = if compact {
            column![timeline, inspector].spacing(12).into()
        } else {
            row![timeline, inspector].spacing(12).into()
        };

        container(column![header, editor, transport].spacing(12))
            .padding(12)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
