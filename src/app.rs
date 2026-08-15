use crate::{
    Comparison, Project, RenderState, ReviewState, ReviewWorkspace, RunRequest, WorkflowStep,
    run_with_progress, validate,
};
use eframe::egui;
use std::{
    process::{Child, Command},
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
    review: Option<ReviewWorkspace>,
    playback: Vec<Child>,
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
            let stale = matches!(self.project.as_ref().unwrap().render(), RenderState::Stale);
            if stale && self.processing.is_none() && ui.button("Rerender protections").clicked() {
                self.start_processing();
            }
            let mut audition = false;
            if let Some(review) = &mut self.review {
                ui.heading("Synchronized waveforms");
                let width = ui.available_width();
                for (track, source) in self.project.as_ref().unwrap().sources.iter().enumerate() {
                    let highlighted = review.highlighted_tracks().contains(&track);
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(width, 34.0), egui::Sense::hover());
                    let color = if highlighted {
                        egui::Color32::LIGHT_BLUE
                    } else {
                        egui::Color32::DARK_GRAY
                    };
                    ui.painter().rect_filled(rect, 2.0, color);
                    if let Some(points) = review.waveforms.get(track) {
                        let center = rect.center().y;
                        for (index, (min, max)) in points.iter().enumerate() {
                            let x = rect.left()
                                + index as f32 * rect.width() / points.len().max(1) as f32;
                            ui.painter().line_segment(
                                [
                                    egui::pos2(x, center - max * 15.0),
                                    egui::pos2(x, center - min * 15.0),
                                ],
                                egui::Stroke::new(1.0, egui::Color32::WHITE),
                            );
                        }
                    }
                    ui.painter().text(
                        rect.left_center() + egui::vec2(8.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        source
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Playhead {:.2}s",
                        review.playhead() as f32 / 48_000.0
                    ));
                    ui.selectable_value(review.comparison_mut(), Comparison::Original, "Original");
                    ui.add_enabled_ui(!stale, |ui| {
                        ui.selectable_value(
                            review.comparison_mut(),
                            Comparison::Processed,
                            "Processed",
                        );
                    });
                });
                audition = ui.button("Audition selection").clicked();
                ui.heading("Attenuation regions");
                ui.label("Review list");
                let mut warning_select = None;
                for (index, region) in review
                    .regions
                    .iter()
                    .enumerate()
                    .filter(|(_, region)| region.review_warning.is_some())
                {
                    if ui
                        .button(region.review_warning.as_deref().unwrap())
                        .clicked()
                    {
                        warning_select = Some(index);
                    }
                }
                if let Some(index) = warning_select {
                    review.select(index);
                    audition = true;
                }
                let mut select = None;
                for (index, region) in review.regions.iter().enumerate() {
                    let title = format!(
                        "{:.2}–{:.2}s · {} dB · {:?} · {:?} · {}",
                        region.start as f32 / 48_000.0,
                        region.end as f32 / 48_000.0,
                        region.applied_attenuation_db,
                        region.decision,
                        region.evidence_reasons,
                        region.review_warning.as_deref().unwrap_or("")
                    );
                    if ui
                        .selectable_label(review.selected_region() == Some(region), title)
                        .clicked()
                    {
                        select = Some(index);
                    }
                }
                if let Some(index) = select {
                    review.select(index);
                }
                if let Some(region) = review.selected_region().cloned() {
                    ui.horizontal(|ui| {
                        let key = region.key();
                        let state = self
                            .project
                            .as_ref()
                            .unwrap()
                            .review_states()
                            .get(&key)
                            .copied()
                            .unwrap_or(ReviewState::Unreviewed);
                        if ui
                            .button(if state == ReviewState::Reviewed {
                                "Mark unreviewed"
                            } else {
                                "Mark reviewed"
                            })
                            .clicked()
                            && let Err(error) = self.project.as_mut().unwrap().set_review_state(
                                key,
                                if state == ReviewState::Reviewed {
                                    ReviewState::Unreviewed
                                } else {
                                    ReviewState::Reviewed
                                },
                            )
                        {
                            self.messages.push(error.to_string());
                        }
                        if ui.button("Protect this region").clicked() {
                            if let Err(error) = self
                                .project
                                .as_mut()
                                .unwrap()
                                .protect_range(region.start, region.end)
                            {
                                self.messages.push(error.to_string());
                            }
                            review.set_comparison(Comparison::Original);
                        }
                        if let Some(track) = region.affected_track
                            && ui.button(format!("Protect track {}", track + 1)).clicked()
                        {
                            if let Err(error) = self.project.as_mut().unwrap().protect_track(track)
                            {
                                self.messages.push(error.to_string());
                            }
                            review.set_comparison(Comparison::Original);
                        }
                    });
                    if let Some((mut start, mut end)) = review.selection() {
                        ui.horizontal(|ui| {
                            ui.label("Selected range (samples)");
                            ui.add(egui::DragValue::new(&mut start));
                            ui.add(egui::DragValue::new(&mut end));
                            review.set_selection(start, end);
                            if ui.button("Protect selected range").clicked() {
                                if let Err(error) = self
                                    .project
                                    .as_mut()
                                    .unwrap()
                                    .protect_range(start.min(end), start.max(end))
                                {
                                    self.messages.push(error.to_string());
                                }
                                review.set_comparison(Comparison::Original);
                            }
                        });
                    }
                }
            }
            if audition {
                self.audition();
            }
        } else {
            ui.label("Export continues in ticket 06.");
        }
        for message in &self.messages {
            ui.label(message);
        }
    }
}
