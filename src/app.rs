use crate::{Project, RenderState, RunRequest, WorkflowStep, run_with_progress, validate};
use eframe::egui;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver},
    },
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct CleanSlateApp {
    project: Option<Project>,
    messages: Vec<String>,
    processing: Option<Processing>,
}

struct Processing {
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU8>,
    result: Receiver<Result<String, String>>,
}

impl CleanSlateApp {
    fn create_project(&mut self) {
        let stems = rfd::FileDialog::new()
            .add_filter("WAV audio", &["wav"])
            .pick_files();
        let Some(stems) = stems else { return };
        let Some(path) = rfd::FileDialog::new()
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
            .add_filter("CleanSlate project", &["cleanslate"])
            .pick_file()
        else {
            return;
        };
        match Project::open(path) {
            Ok(project) => {
                self.project = Some(project);
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
            }
            Err(error) => self.messages = vec![error],
        }
    }
}

impl eframe::App for CleanSlateApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_processing();
        if self.processing.is_some() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }

        ui.horizontal(|ui| {
            if ui.button("New project").clicked() {
                self.create_project();
            }
            if ui.button("Open project").clicked() {
                self.open_project();
            }
        });
        ui.heading("CleanSlate");
        let steps = [
            WorkflowStep::Import,
            WorkflowStep::Validate,
            WorkflowStep::Process,
            WorkflowStep::Review,
            WorkflowStep::Export,
        ];
        let current = self
            .project
            .as_ref()
            .map_or(WorkflowStep::Import, Project::step);
        ui.horizontal(|ui| {
            for step in steps {
                let _ = ui.selectable_label(step == current, format!("{step:?}"));
            }
        });
        ui.separator();

        let sources = self
            .project
            .as_ref()
            .map(|project| project.sources.clone())
            .unwrap_or_default();
        for (index, source) in sources.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(source.path.display().to_string());
                if !source.available && ui.button("Relink").clicked() {
                    let replacement = rfd::FileDialog::new()
                        .add_filter("WAV audio", &["wav"])
                        .pick_file();
                    if let Some(path) = replacement {
                        match self.project.as_mut().unwrap().relink(index, path) {
                            Ok(()) => {}
                            Err(error) => self.messages = vec![error.to_string()],
                        }
                    }
                }
            });
        }

        ui.add_space(12.0);
        if self.project.is_none() || current == WorkflowStep::Import {
            if ui.button("Import stems").clicked() {
                self.create_project();
            }
        } else if current == WorkflowStep::Validate {
            if ui.button("Validate session").clicked() {
                self.validate();
            }
        } else if current == WorkflowStep::Process {
            if let Some(processing) = &self.processing {
                ui.spinner();
                ui.label(format!(
                    "Processing… {}%",
                    processing.progress.load(Ordering::Relaxed)
                ));
                if ui.button("Cancel processing").clicked() {
                    processing.cancelled.store(true, Ordering::Relaxed);
                }
            } else if ui.button("Process session").clicked() {
                self.start_processing();
            }
        } else if current == WorkflowStep::Review {
            ui.label(match self.project.as_ref().unwrap().render() {
                RenderState::Current { .. } => "Render current",
                RenderState::Stale => "Render stale",
                RenderState::None => "No render",
            });
            ui.label("Review workspace continues in ticket 05.");
        } else {
            ui.label("Export continues in ticket 06.");
        }
        for message in &self.messages {
            ui.label(message);
        }
    }
}
