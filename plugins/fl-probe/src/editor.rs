use crate::{
    FlProbe, ListenMode, ProbeParams,
    calibration::{Label, ReferenceFrame, TrainingTask},
    history::{BINS, HistorySnapshot},
    runtime::GuiState,
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use nice_plug::{
    context::gui::{GuiContext, ParamSetter},
    prelude::*,
};
use nice_plug_egui::NiceEguiApp;
use std::{
    sync::{Arc, atomic::Ordering::Relaxed},
    time::Duration,
};

// Match src/appearance.rs. The plugin owns its renderer, not a separate visual identity.
const SURFACE: Color32 = Color32::from_rgb(32, 34, 38);
const PANEL: Color32 = Color32::from_rgb(41, 45, 51);
const TEXT: Color32 = Color32::from_rgb(228, 231, 235);
const MUTED: Color32 = Color32::from_rgb(173, 181, 192);
const TEAL: Color32 = Color32::from_rgb(111, 190, 183);
const BLUE: Color32 = Color32::from_rgb(119, 155, 199);
const BORDER: Color32 = Color32::from_rgb(61, 67, 76);
const ERROR: Color32 = Color32::from_rgb(219, 139, 139);

pub struct Panel {
    session_ui: crate::session_editor::SessionUi,
    params: Arc<ProbeParams>,
    executor: AsyncExecutor<FlProbe>,
    state: Arc<GuiState>,
    context: Option<GuiContext>,
    workspace: Workspace,
}

#[derive(Default)]
struct Workspace {
    history: HistorySnapshot,
    frozen: bool,
    selection: Option<(i64, i64)>,
    drag_anchor: Option<i64>,
    settings_open: bool,
    last_error: Option<String>,
    session_id: usize,
}

pub(crate) fn style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = PANEL;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = Color32::from_rgb(58, 85, 125);
    visuals.selection.stroke.color = Color32::from_rgb(160, 185, 255);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(52, 57, 64);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 71, 80);
    visuals.widgets.active.bg_fill = Color32::from_rgb(58, 85, 125);
    context.set_visuals(visuals);
    context.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.slider_width = 160.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(12.0));
    });
}

fn set<P: Param>(setter: Option<&ParamSetter<'_>>, param: &P, value: P::Plain) {
    if let Some(setter) = setter {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, value);
        setter.end_set_parameter(param);
    }
}

impl Panel {
    pub(crate) fn new(
        params: Arc<ProbeParams>,
        executor: AsyncExecutor<FlProbe>,
        state: Arc<GuiState>,
    ) -> Self {
        Self {
            session_ui: crate::session_editor::SessionUi::default(),
            params,
            executor,
            state,
            context: None,
            workspace: Workspace::default(),
        }
    }
}

impl NiceEguiApp for Panel {
    fn build(
        &mut self,
        context: egui::Context,
        gui_context: GuiContext,
        _: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        style(&context);
        self.context = Some(gui_context);
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.context = None;
        self.session_ui.editor_closed();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut nice_plug_egui::Frame) {
        self.show(ui);
    }
}

impl Panel {
    fn show(&mut self, ui: &mut egui::Ui) {
        let Some(context) = self.context.clone() else {
            return;
        };
        let setter = context.param_setter();
        if cfg!(cleanslate_podcast) || self.params.session_mode.value() {
            self.session_ui
                .show(ui, &self.params, Some(&setter), &self.state);
            let interval = if self.state.playing.load(Relaxed) {
                16
            } else {
                100
            };
            ui.ctx()
                .request_repaint_after(Duration::from_millis(interval));
            return;
        }
        if ui.button("Open the persistent Session workspace").clicked() {
            set(Some(&setter), &self.params.session_mode, true);
        }
        if let Some(task) = self
            .workspace
            .show(ui, &self.params, Some(&setter), &self.state)
        {
            self.executor.execute_background(task);
        }
        // No repaint clock is installed while the native editor is closed.
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
}

impl Workspace {
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        params: &ProbeParams,
        setter: Option<&ParamSetter<'_>>,
        state: &GuiState,
    ) -> Option<TrainingTask> {
        let session = state.session.lock().clone();
        let identity = session
            .as_ref()
            .map_or(0, |session| Arc::as_ptr(session) as usize);
        if identity != self.session_id {
            self.history = HistorySnapshot::default();
            self.frozen = false;
            self.selection = None;
            self.last_error = None;
            self.session_id = identity;
        }
        if let Some(session) = &session {
            let live = session.history.snapshot();
            if !self.frozen {
                self.history = live;
            }
        }
        let view = session.as_ref().map(|session| session.snapshot());
        let busy = view.as_ref().is_some_and(|view| view.busy);
        let usable = view
            .as_ref()
            .is_some_and(|view| view.trained && view.confirmed && !view.busy);
        let can_confirm = params.saved.lock().as_ref().is_some_and(|document| {
            params
                .saved_examples
                .lock()
                .as_ref()
                .is_none_or(|rows| *rows == document.rows)
        });
        let labels = session
            .as_ref()
            .map_or_else(Vec::new, |session| session.labels());
        let stale = self.frozen
            && session
                .as_ref()
                .is_some_and(|session| session.history.latest_epoch() != self.history.epoch);
        let mut learn = None;

        egui::Frame::new()
            .fill(SURFACE)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CleanSlate").size(23.0).strong());
                    ui.label(
                        egui::RichText::new(concat!(
                            "Listening prototype · ",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .small()
                        .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(format!(
                                "Group {} · Mic {}",
                                params.group.value(),
                                params.member.value()
                            ))
                            .clicked()
                        {
                            self.settings_open = !self.settings_open;
                        }
                        for (mode, name) in [
                            (ListenMode::Removed, "Removed"),
                            (ListenMode::Processed, "Processed"),
                            (ListenMode::Original, "Original"),
                        ] {
                            if ui
                                .selectable_label(params.listen.value() == mode, name)
                                .clicked()
                            {
                                set(setter, &params.listen, mode);
                            }
                        }
                    });
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.strong("Waveform history");
                    ui.label(
                        egui::RichText::new(if self.frozen {
                            "Frozen view"
                        } else {
                            "Following host playback"
                        })
                        .color(if self.frozen { BLUE } else { TEAL }),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.history.frames.is_empty(),
                                egui::Button::new(if self.frozen {
                                    "Resume live"
                                } else {
                                    "Freeze history"
                                }),
                            )
                            .clicked()
                        {
                            self.frozen = !self.frozen;
                            self.selection = None;
                            self.last_error = None;
                        }
                        ui.label(
                            egui::RichText::new("Input · shared time · auto scale")
                                .small()
                                .color(MUTED),
                        );
                    });
                });
                let height = (ui.available_height() - 210.0).max(190.0);
                self.waveforms(ui, height, &labels);
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    let selected = self.selection.map(|(a, b)| ordered(a, b));
                    if let Some((a, b)) = selected {
                        let rate = self
                            .history
                            .frames
                            .first()
                            .map_or(48000.0, |frame| f32::from_bits(frame.reference.rate));
                        let count = self
                            .history
                            .frames
                            .iter()
                            .filter(|frame| frame.reference.start >= a && frame.reference.end <= b)
                            .count();
                        ui.label(format!(
                            "{:.2} s usable · {} paired frames",
                            count as f32 * 0.032,
                            count
                        ));
                        ui.label(
                            egui::RichText::new(format!("{}–{}", time(a, rate), time(b, rate)))
                                .small()
                                .color(MUTED),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(if self.history.frames.is_empty() {
                                "Play both microphone tracks in your DAW to fill this history."
                            } else if self.frozen {
                                "Drag across either waveform to select an example."
                            } else {
                                "Freeze the view, then select and label a past section."
                            })
                            .color(MUTED),
                        );
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    let ready = self.frozen && self.selection.is_some() && !busy && !stale;
                    for (label, name) in [
                        (Label::Room, "Room"),
                        (Label::Member1, "Mic 1"),
                        (Label::Member2, "Mic 2"),
                        (Label::Both, "Both"),
                        (Label::None, "Erase label"),
                    ] {
                        if ui.add_enabled(ready, egui::Button::new(name)).clicked()
                            && let (Some(session), Some((a, b))) = (&session, self.selection)
                        {
                            let (a, b) = ordered(a, b);
                            self.last_error =
                                session.label_history(&self.history, a, b, label).err();
                        }
                    }
                    ui.separator();
                    let enough = view
                        .as_ref()
                        .is_some_and(|view| view.counts[..3].iter().all(|&n| n >= 10));
                    if ui
                        .add_enabled(
                            enough && !busy,
                            egui::Button::new(if busy {
                                "Learning…"
                            } else {
                                "Learn examples"
                            })
                            .fill(BLUE),
                        )
                        .clicked()
                        && let Some(session) = &session
                    {
                        self.last_error = None;
                        learn = Some(TrainingTask::new(session.clone()));
                    }
                });
                if let Some(view) = &view {
                    ui.label(
                        egui::RichText::new(format!(
                            "Room {:.2}s  ·  Mic 1 {:.2}s  ·  Mic 2 {:.2}s  ·  Both {:.2}s",
                            view.counts[0] as f32 * 0.032,
                            view.counts[1] as f32 * 0.032,
                            view.counts[2] as f32 * 0.032,
                            view.counts[3] as f32 * 0.032
                        ))
                        .small()
                        .color(MUTED),
                    );
                }
                ui.add_space(7.0);
                ui.separator();
                ui.horizontal(|ui| {
                    let mut enabled = params.enabled.value();
                    if ui
                        .add_enabled(
                            usable || enabled,
                            egui::Checkbox::new(&mut enabled, "Enable processing"),
                        )
                        .changed()
                    {
                        set(setter, &params.enabled, enabled);
                    }
                    ui.separator();
                    ui.label("Reduction");
                    let mut reduction = params.reduction.value();
                    if ui
                        .add(egui::Slider::new(&mut reduction, -48.0..=0.0).suffix(" dB"))
                        .changed()
                    {
                        set(setter, &params.reduction, reduction);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gain = f32::from_bits(state.gain.load(Relaxed)).clamp(0.00001, 1.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.1} dB applied",
                                (-20.0 * gain.log10()).abs()
                            ))
                            .color(TEAL),
                        );
                    });
                });
                let error = self
                    .last_error
                    .clone()
                    .or_else(|| state.error.lock().clone());
                let status = if stale {
                    "Playback moved. Resume live history before labeling a new section."
                } else if let Some(error) = error.as_deref() {
                    error
                } else if session.is_none() {
                    "Connect the two microphones to one group in settings to begin."
                } else if busy {
                    "Learning your labeled examples…"
                } else if usable {
                    state.status_text()
                } else if view
                    .as_ref()
                    .is_some_and(|view| view.counts[..3].iter().all(|&n| n >= 10))
                {
                    "Examples ready. Learn to enable calibrated processing."
                } else {
                    "Label at least 0.32 s each of Room, Mic 1 and Mic 2, then Learn."
                };
                ui.label(egui::RichText::new(status).size(13.0).color(
                    if error.is_some() || stale {
                        ERROR
                    } else {
                        MUTED
                    },
                ));
                if session.is_none() && ui.button("Connect microphones").clicked() {
                    self.settings_open = true;
                }
                if let (Some(session), Some(view)) = (&session, &view)
                    && view.trained
                    && !view.confirmed
                    && can_confirm
                    && ui.small_button("Confirm saved calibration").clicked()
                {
                    self.last_error = session.confirm_saved().err();
                }
            });
        let mut open = self.settings_open;
        egui::Window::new("Session & settings")
            .open(&mut open)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Use one group and a different member on each track.");
                ui.horizontal(|ui| {
                    ui.label("Group");
                    let mut group = params.group.value();
                    if ui
                        .add(egui::DragValue::new(&mut group).range(0..=8))
                        .changed()
                    {
                        set(setter, &params.group, group);
                    }
                    ui.label("Microphone");
                    let mut member = params.member.value();
                    if ui
                        .add(egui::DragValue::new(&mut member).range(1..=2))
                        .changed()
                    {
                        set(setter, &params.member, member);
                    }
                });
                ui.label(
                    egui::RichText::new(
                        "Group changes and enabling processing may require host reactivation.",
                    )
                    .small()
                    .color(MUTED),
                );
                ui.separator();
                for (name, param, min, max) in [
                    ("Fade in", &params.attack, 5.0, 80.0),
                    ("Fade out", &params.release, 10.0, 150.0),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        let mut value = param.value();
                        if ui
                            .add(egui::Slider::new(&mut value, min..=max).suffix(" ms"))
                            .changed()
                        {
                            set(setter, param, value);
                        }
                    });
                }
                let latency = f32::from_bits(state.latency_ms.load(Relaxed));
                ui.label(format!("Reported delay: {latency:.1} ms"));
                ui.label("Freeze changes the view only. Transport stays in your DAW.");
                ui.label("Recalibrate after changing recordings, microphones or gain.");
                if let Some(view) = &view {
                    ui.label(&view.message);
                }
                if ui
                    .add_enabled(!busy, egui::Button::new("Clear all examples"))
                    .clicked()
                    && let Some(session) = &session
                {
                    self.last_error = session.try_clear().err();
                }
                ui.collapsing("Diagnostics", |ui| {
                    let mut export = params.export_run.value();
                    if ui
                        .checkbox(&mut export, "Label this run as Export")
                        .changed()
                    {
                        set(setter, &params.export_run, export);
                    }
                    ui.label("VST3 is the listening target. CLAP attenuation is disabled.");
                    ui.label("Streaming features differ from the desktop analysis pipeline.");
                });
            });
        self.settings_open = open;
        learn
    }

    fn waveforms(&mut self, ui: &mut egui::Ui, height: f32, labels: &[ReferenceFrame]) {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, Color32::from_rgb(24, 27, 31));
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
        let plot = Rect::from_min_max(
            rect.min + egui::vec2(74.0, 54.0),
            rect.max - egui::vec2(12.0, 25.0),
        );
        if self.history.frames.is_empty() {
            painter.text(
                rect.center() - egui::vec2(0.0, 13.0),
                Align2::CENTER_CENTER,
                "Your microphone history will appear here",
                FontId::proportional(19.0),
                TEXT,
            );
            painter.text(
                rect.center() + egui::vec2(0.0, 17.0),
                Align2::CENTER_CENTER,
                "Play in the DAW. Then freeze, select and label.",
                FontId::proportional(14.0),
                MUTED,
            );
            return;
        }
        let first = self.history.frames.first().unwrap().reference;
        let last = self.history.frames.last().unwrap().reference;
        let rate = f32::from_bits(first.rate);
        let span = last
            .end
            .saturating_sub(first.start)
            .clamp((rate * 3.0) as i64, (rate * 30.0) as i64);
        let left = last.end.saturating_sub(span);
        let right = last.end;
        let x = |sample: i64| {
            plot.left()
                + ((i128::from(sample) - i128::from(left)) as f64 / span as f64) as f32
                    * plot.width()
        };
        for step in 0..=6 {
            let xx = plot.left() + plot.width() * step as f32 / 6.0;
            painter.line_segment(
                [Pos2::new(xx, plot.top()), Pos2::new(xx, plot.bottom())],
                Stroke::new(1.0, Color32::from_rgb(42, 47, 54)),
            );
            painter.text(
                Pos2::new(xx, rect.bottom() - 11.0),
                if step == 0 {
                    Align2::LEFT_CENTER
                } else if step == 6 {
                    Align2::RIGHT_CENTER
                } else {
                    Align2::CENTER_CENTER
                },
                time(left + span * step / 6, rate),
                FontId::monospace(11.0),
                MUTED,
            );
        }
        let lane_height = plot.height() / 2.0;
        let scale = self
            .history
            .frames
            .iter()
            .flat_map(|f| f.peaks.iter().flatten())
            .fold(0.03_f32, |a, p| a.max(p.min.abs()).max(p.max.abs()));
        let pixels = plot.width().ceil().clamp(1.0, 4096.0) as usize;
        for mic in 0..2 {
            let center = plot.top() + lane_height * (mic as f32 + 0.5);
            painter.text(
                Pos2::new(rect.left() + 13.0, center - 8.0),
                Align2::LEFT_CENTER,
                format!("Mic {}", mic + 1),
                FontId::proportional(14.0),
                TEXT,
            );
            painter.text(
                Pos2::new(rect.left() + 13.0, center + 12.0),
                Align2::LEFT_CENTER,
                "Input",
                FontId::proportional(11.0),
                MUTED,
            );
            painter.line_segment(
                [
                    Pos2::new(plot.left(), center),
                    Pos2::new(plot.right(), center),
                ],
                Stroke::new(1.0, BORDER),
            );
            let mut columns = vec![None::<(f32, f32)>; pixels];
            for frame in &self.history.frames {
                let r = frame.reference;
                if r.end <= left || r.start >= right {
                    continue;
                }
                for (bin, peak) in frame.peaks[mic].iter().enumerate() {
                    let start = r.start + (r.end - r.start) * bin as i64 / BINS as i64;
                    let end = r.start + (r.end - r.start) * (bin + 1) as i64 / BINS as i64;
                    if end <= left || start >= right {
                        continue;
                    }
                    let a = ((x(start) - plot.left()).floor() as isize)
                        .clamp(0, pixels as isize - 1) as usize;
                    let b = ((x(end) - plot.left()).ceil() as isize).clamp(0, pixels as isize - 1)
                        as usize;
                    for column in &mut columns[a..=b] {
                        *column = Some(column.map_or((peak.min, peak.max), |(min, max)| {
                            (min.min(peak.min), max.max(peak.max))
                        }));
                    }
                }
            }
            let color = if mic == 0 { TEAL } else { BLUE };
            for (column, peak) in columns.iter().enumerate() {
                if let Some((min, max)) = peak {
                    let xx = plot.left() + column as f32 * plot.width() / pixels as f32;
                    let amplitude = lane_height * 0.35 / scale;
                    painter.line_segment(
                        [
                            Pos2::new(xx, center - max * amplitude),
                            Pos2::new(xx, center - min * amplitude),
                        ],
                        Stroke::new(1.1, color),
                    );
                }
            }
        }
        let mut ranges: Vec<(i64, i64, Label)> = Vec::new();
        for label in labels {
            if label.end <= left || label.start >= right {
                continue;
            }
            if let Some(previous) = ranges.last_mut()
                && previous.1 == label.start
                && previous.2 == label.label
            {
                previous.1 = label.end;
            } else {
                ranges.push((label.start, label.end, label.label));
            }
        }
        for (start, end, label) in ranges {
            let (name, color) = match label {
                Label::Room => ("Room", MUTED),
                Label::Member1 => ("Mic 1", TEAL),
                Label::Member2 => ("Mic 2", BLUE),
                Label::Both => ("Both", Color32::from_rgb(182, 148, 213)),
                Label::None => continue,
            };
            let badge = Rect::from_min_max(
                Pos2::new(x(start.max(left)), plot.top() - 23.0),
                Pos2::new(x(end.min(right)), plot.top() - 5.0),
            );
            painter.rect_filled(badge, 2.0, color.gamma_multiply(0.28));
            painter.line_segment(
                [badge.left_bottom(), badge.right_bottom()],
                Stroke::new(2.0, color),
            );
            if badge.width() >= 24.0 {
                painter.with_clip_rect(badge).text(
                    badge.center(),
                    Align2::CENTER_CENTER,
                    name,
                    FontId::proportional(11.0),
                    TEXT,
                );
            }
        }
        if let Some((a, b)) = self.selection {
            let (a, b) = ordered(a, b);
            let a = x(a).clamp(plot.left(), plot.right());
            let b = x(b).clamp(plot.left(), plot.right());
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(a, plot.top()), Pos2::new(b, plot.bottom())),
                0.0,
                Color32::from_rgba_unmultiplied(95, 150, 240, 40),
            );
            for xx in [a, b] {
                painter.line_segment(
                    [Pos2::new(xx, plot.top()), Pos2::new(xx, plot.bottom())],
                    Stroke::new(1.5, Color32::from_rgb(160, 185, 255)),
                );
            }
        }
        painter.text(
            Pos2::new(plot.left(), rect.top() + 13.0),
            Align2::LEFT_CENTER,
            "Input waveforms · labeled examples",
            FontId::proportional(10.0),
            MUTED,
        );
        if self.frozen {
            response
                .clone()
                .on_hover_cursor(egui::CursorIcon::Crosshair);
            let sample_at = |position: Pos2| {
                left + (((position.x - plot.left()) / plot.width()).clamp(0.0, 1.0) * span as f32)
                    as i64
            };
            if response.drag_started()
                && let Some(position) = ui.input(|input| input.pointer.press_origin())
            {
                let sample = sample_at(position);
                self.drag_anchor = Some(sample);
                self.selection = Some((sample, sample));
            }
            if response.dragged()
                && let (Some(anchor), Some(position)) =
                    (self.drag_anchor, response.interact_pointer_pos())
            {
                self.selection = Some((anchor, sample_at(position)));
            }
            if response.drag_stopped() {
                self.drag_anchor = None;
            }
            if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.selection = None;
            }
        }
    }
}

fn ordered(a: i64, b: i64) -> (i64, i64) {
    (a.min(b), a.max(b))
}
fn time(sample: i64, rate: f32) -> String {
    let seconds = sample as f64 / f64::from(rate);
    let sign = if seconds < 0.0 { "−" } else { "" };
    let seconds = seconds.abs();
    format!("{sign}{}:{:04.1}", (seconds / 60.0) as u64, seconds % 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nice_plug::params::InternalParamMut;
    use nice_plug::{
        context::{PluginApi, gui::GuiContextInner},
        params::internals::ParamPtr,
        plugin::PluginState,
    };

    struct IdleHost;

    impl GuiContextInner for IdleHost {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Clap
        }
        unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {
            panic!("Rendering must not begin host automation");
        }
        unsafe fn raw_set_parameter_normalized(&self, _: ParamPtr, _: f32) {
            panic!("Rendering must not change host parameters");
        }
        unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {
            panic!("Rendering must not end host automation");
        }
        fn get_state(&self) -> PluginState {
            panic!("Rendering must not serialize host state");
        }
        fn set_state(&self, _: PluginState) {
            panic!("Rendering must not restore host state");
        }
        fn request_restart(&self) {
            panic!("Rendering must not restart the host");
        }
    }

    #[test]
    fn actual_panel_stops_polling_while_hidden_and_resumes_visible_polling() {
        for session_mode in [false, true] {
            for playing in [false, true] {
                let probe = FlProbe::default();
                // The test owns the parameters and uses a declared boolean value.
                unsafe {
                    probe
                        .params
                        .session_mode
                        ._internal_set_plain_value(session_mode)
                };
                let state = Arc::new(GuiState::new());
                state.playing.store(playing, Relaxed);
                let mut panel = Panel::new(
                    probe.params.clone(),
                    AsyncExecutor::new(
                        Arc::new(|_| panic!("Rendering must not schedule training")),
                        Arc::new(|_| panic!("Rendering must not schedule host tasks")),
                    ),
                    state,
                );
                panel.context = Some(GuiContext::new(Arc::new(IdleHost)));
                let context = egui::Context::default();
                style(&context);
                let mut now = 0.0;
                let mut frame = |panel: &mut Panel, hidden| {
                    now += 1.0;
                    let mut output = context.run_ui(
                        egui::RawInput {
                            time: Some(now),
                            screen_rect: Some(Rect::from_min_size(
                                Pos2::ZERO,
                                egui::vec2(1000.0, 680.0),
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            if hidden {
                                ui.set_invisible();
                            }
                            panel.show(ui);
                        },
                    );
                    output.textures_delta.clear();
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                };
                let polling = Duration::from_millis(if cfg!(cleanslate_podcast) || session_mode {
                    if playing { 16 } else { 100 }
                } else {
                    50
                });
                for _ in 0..3 {
                    frame(&mut panel, false);
                }
                assert_eq!(frame(&mut panel, false), polling);
                panel.workspace.drag_anchor = Some(12);
                panel.workspace.selection = Some((12, 24));
                panel.workspace.frozen = true;
                for _ in 0..3 {
                    frame(&mut panel, true);
                }
                assert!(
                    frame(&mut panel, true) > Duration::from_secs(1),
                    "Hidden native adapter must stop its playing/idle polling clock"
                );
                assert_eq!(panel.workspace.drag_anchor, None);
                assert_eq!(panel.workspace.selection, Some((12, 24)));
                assert!(panel.workspace.frozen);
                for _ in 0..3 {
                    frame(&mut panel, false);
                }
                assert_eq!(frame(&mut panel, false), polling);
            }
        }
    }

    #[test]
    fn editor_bundles_readable_fonts_without_host_resources() {
        assert!(!egui::FontDefinitions::default().font_data.is_empty());
        assert_eq!(ordered(12, 4), (4, 12));
        assert_eq!(time(48000, 48000.0), "0:01.0");
        let context = egui::Context::default();
        style(&context);
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            ui.heading("CleanSlate");
            ui.label("Freeze history · Label example");
        });
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
    }

    fn fixture() -> (Arc<ProbeParams>, Arc<crate::calibration::Session>, GuiState) {
        let params = FlProbe::default().params.clone();
        // The test owns these parameters; values stay in their declared ranges.
        unsafe {
            params.group._internal_set_plain_value(1);
            params.member._internal_set_plain_value(2);
        }
        let session = Arc::new(crate::calibration::Session::new());
        session.bind(0, &params, 48000_f32.to_bits(), 2).unwrap();
        for index in 0..720 {
            let label = match (index / 120) % 4 {
                0 => Label::Room,
                1 => Label::Member1,
                2 => Label::Member2,
                _ => Label::Both,
            };
            let (a, b) = crate::calibration::tests::signal(label, index as usize, 2);
            let reference = ReferenceFrame {
                start: index * 1536,
                end: (index + 1) * 1536,
                label: Label::None,
                rate: 48000_f32.to_bits(),
                channels: 2,
                features: crate::calibration::acoustic_features(&a, &b, 2).unwrap(),
            };
            session
                .history
                .push(crate::history::HistoryFrame::from_audio(
                    1, reference, &a, &b, 2,
                ));
        }
        let snapshot = session.history.snapshot();
        for (start, end, label) in [
            (12 * 1536, 42 * 1536, Label::Room),
            (130 * 1536, 170 * 1536, Label::Member1),
            (250 * 1536, 285 * 1536, Label::Member2),
            (370 * 1536, 400 * 1536, Label::Both),
        ] {
            session.label_history(&snapshot, start, end, label).unwrap();
        }
        let state = GuiState::new();
        *state.session.lock() = Some(session.clone());
        (params, session, state)
    }

    #[test]
    fn actual_workspace_renders_at_compact_and_full_sizes() {
        let (params, session, state) = fixture();
        for (name, width, height, loaded) in [
            ("empty", 1000, 680, false),
            ("history", 1000, 680, true),
            ("compact", 760, 620, true),
        ] {
            let context = egui::Context::default();
            style(&context);
            let state_ref = if loaded { &state } else { &GuiState::new() };
            let mut workspace = Workspace::default();
            if loaded {
                workspace.session_id = Arc::as_ptr(&session) as usize;
                workspace.history = session.history.snapshot();
                workspace.frozen = true;
                workspace.selection = Some((430 * 1536, 470 * 1536));
            }
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(width as f32, height as f32),
                )),
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                workspace.show(ui, &params, None, state_ref);
                assert!(
                    ui.min_rect().bottom() <= height as f32 + 1.0,
                    "{name}: controls overflow window: {:?}",
                    ui.min_rect()
                );
            });
            assert!(!output.shapes.is_empty());
            if let Ok(directory) = std::env::var("CLEANSLATE_PLUGIN_CAPTURES") {
                std::fs::create_dir_all(&directory).unwrap();
                crate::editor_capture::write_capture(
                    &context,
                    &mut output,
                    &std::path::Path::new(&directory).join(format!("{name}.ppm")),
                    width,
                    height,
                )
                .unwrap();
            } else {
                output.textures_delta.clear();
            }
        }
    }

    #[test]
    fn frozen_canvas_drag_selects_both_lanes_without_moving_history() {
        let (_params, session, _state) = fixture();
        let mut workspace = Workspace {
            history: session.history.snapshot(),
            frozen: true,
            ..Default::default()
        };
        let before = workspace.history.frames[0].reference.start;
        let context = egui::Context::default();
        style(&context);
        let events = [
            vec![],
            vec![
                egui::Event::PointerMoved(egui::pos2(150.0, 100.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(150.0, 100.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
            vec![egui::Event::PointerMoved(egui::pos2(450.0, 220.0))],
            vec![egui::Event::PointerButton {
                pos: egui::pos2(450.0, 220.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        ];
        for events in events {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 400.0))),
                    events,
                    ..Default::default()
                },
                |ui| workspace.waveforms(ui, 300.0, &[]),
            );
            output.textures_delta.clear();
        }
        let (a, b) = workspace.selection.unwrap();
        assert!(b > a);
        assert_eq!(workspace.history.frames[0].reference.start, before);
        session
            .label_history(&workspace.history, a, b, Label::Member1)
            .unwrap();
        assert!(!session.labels().is_empty());
    }
}
