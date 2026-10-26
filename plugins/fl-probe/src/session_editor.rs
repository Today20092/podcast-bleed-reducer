//! Source-time Session workspace. Transport remains in the host.
use crate::{
    ListenMode, ProbeParams,
    runtime::{GuiState, RuntimeStatus},
    session_mode::SessionView,
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use nice_plug::{context::gui::ParamSetter, prelude::*};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TEXT: Color32 = Color32::from_rgb(228, 231, 235);
const MUTED: Color32 = Color32::from_rgb(173, 181, 192);
const TEAL: Color32 = Color32::from_rgb(111, 190, 183);
const BLUE: Color32 = Color32::from_rgb(119, 155, 199);
const BORDER: Color32 = Color32::from_rgb(61, 67, 76);
const ERROR: Color32 = Color32::from_rgb(219, 139, 139);

#[derive(Clone, Copy)]
enum Navigation {
    Fit,
    Selection,
    In,
    Out,
    Earlier,
    Later,
    Center,
}
impl Navigation {
    const ALL: [Self; 7] = [
        Self::Fit,
        Self::Selection,
        Self::Out,
        Self::In,
        Self::Earlier,
        Self::Later,
        Self::Center,
    ];
    fn control(self) -> (&'static str, &'static str, egui::Key) {
        match self {
            Self::Fit => (
                "Fit captured",
                "Fit captured history · Home",
                egui::Key::Home,
            ),
            Self::Selection => ("Fit selection", "Fit selected range · F", egui::Key::F),
            Self::Out => ("Zoom out", "Zoom out · Minus", egui::Key::Minus),
            Self::In => ("Zoom in", "Zoom in · Plus", egui::Key::Plus),
            Self::Earlier => ("Earlier", "Pan earlier · Left", egui::Key::ArrowLeft),
            Self::Later => ("Later", "Pan later · Right", egui::Key::ArrowRight),
            Self::Center => ("Center", "Center on DAW playhead · C", egui::Key::C),
        }
    }
}

#[derive(Default)]
pub struct SessionUi {
    capture_id: usize,
    viewport: Option<(i64, i64)>,
    selection: Option<(i64, i64)>,
    drag_anchor: Option<i64>,
    overlap: u8,
    clear_armed: bool,
    error: Option<String>,
    last_view: Option<SessionView>,
    last_query: Option<(Option<(i64, i64)>, usize)>,
    last_refresh: Option<Instant>,
    pan_anchor: Option<(f32, i64, i64)>,
    range_zoom: bool,
    zoom_range: Option<(i64, i64)>,
    playhead: Option<i64>,
    follow: u8,
    manual_navigation: Option<Instant>,
}

fn set<P: Param>(setter: Option<&ParamSetter<'_>>, parameter: &P, value: P::Plain) {
    if let Some(setter) = setter {
        setter.begin_set_parameter(parameter);
        setter.set_parameter(parameter, value);
        setter.end_set_parameter(parameter);
    }
}

fn time(sample: i64, rate: u32) -> String {
    let seconds = sample as f64 / f64::from(rate.max(1));
    let magnitude = seconds.abs();
    format!(
        "{}{:02}:{:05.2}",
        if seconds < 0.0 { "−" } else { "" },
        (magnitude / 60.0).floor() as u64,
        magnitude % 60.0,
    )
}

fn label_name(mask: u8) -> String {
    if mask == 0 {
        "Room".into()
    } else {
        (0..8)
            .filter(|&index| mask & (1_u8 << index) != 0)
            .map(|index| format!("Mic {}", index + 1))
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

fn ordered(a: i64, b: i64) -> Option<(i64, i64)> {
    (a != b).then_some((a.min(b), a.max(b)))
}

fn sample_at(x: f32, plot: Rect, start: i64, end: i64) -> i64 {
    let fraction = ((x - plot.left()) / plot.width().max(1.0)).clamp(0.0, 1.0);
    let offset = (f64::from(fraction) * (i128::from(end) - i128::from(start)) as f64).round();
    (i128::from(start) + offset as i128).clamp(i128::from(start), i128::from(end)) as i64
}

impl SessionUi {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        params: &ProbeParams,
        setter: Option<&ParamSetter<'_>>,
        state: &GuiState,
    ) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(32, 34, 38));
        let capture = state.capture.lock().clone();
        let identity = capture
            .as_ref()
            .map_or(0, |capture| Arc::as_ptr(capture) as usize);
        if self.capture_id != identity {
            *self = Self {
                capture_id: identity,
                ..Self::default()
            };
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(if cfg!(cleanslate_podcast) {
                    "CleanSlate Podcast"
                } else {
                    "CleanSlate"
                })
                .size(20.0)
                .strong(),
            );
            ui.label(egui::RichText::new("Session").color(TEAL));
            ui.separator();
            for (mode, title) in [
                (ListenMode::Original, "Original"),
                (ListenMode::Processed, "Processed"),
                (ListenMode::Removed, "Removed"),
            ] {
                if ui
                    .selectable_label(params.listen.value() == mode, title)
                    .clicked()
                {
                    set(setter, &params.listen, mode);
                }
            }
        });
        #[cfg(cleanslate_podcast)]
        if *params.product_state_rejected.lock() {
            ui.colored_label(ERROR, "State was not loaded: this is not supported Podcast state. The instance was disconnected. Reopen the original project with its original plugin; configure Podcast separately.");
        }
        let mut reassigned = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Group");
            let mut group = params.group.value();
            if ui.add(egui::DragValue::new(&mut group).range(0..=8))
                .on_hover_text("Use the same group on every participating microphone. Zero disconnects this instance.")
                .changed()
            {
                set(setter, &params.group, group);
                reassigned = true;
            }
            ui.label("This microphone");
            let mut member = params.member.value();
            if ui.add(egui::DragValue::new(&mut member).range(1..=8))
                .on_hover_text("Use a different microphone number on each track.")
                .changed()
            {
                set(setter, &params.member, member);
                reassigned = true;
            }
            ui.separator();
            ui.label(egui::RichText::new("Play in your DAW to capture input.").color(MUTED));
        });
        ui.add_space(6.0);
        if reassigned {
            self.viewport = None;
            self.selection = None;
            return;
        }
        if state.status.load(std::sync::atomic::Ordering::Relaxed)
            == RuntimeStatus::Restarting as u8
        {
            self.empty(
                ui,
                "Waiting for the host to reconnect",
                "Save your project and reload the plugin or reopen the project if the host does not reconnect.",
            );
            return;
        }
        let Some(capture) = capture else {
            if let Some(error) = state.error.lock().as_ref() {
                self.empty(ui, "Cannot connect this Session", error);
            } else {
                self.empty(
                    ui,
                    "Connect microphone tracks",
                    "Choose one group and a unique microphone number on each instance.",
                );
            }
            return;
        };
        let columns = (ui.available_width() - 116.0).ceil().clamp(64.0, 2048.0) as usize;
        let position = state.position.load(std::sync::atomic::Ordering::Relaxed);
        self.playhead = (position >= 0).then_some(position);
        if state.playing.load(std::sync::atomic::Ordering::Relaxed)
            && self
                .manual_navigation
                .is_none_or(|last| last.elapsed() > Duration::from_secs(1))
            && self.drag_anchor.is_none()
            && self.pan_anchor.is_none()
            && let (Some(position), Some((start, end)), Some(last)) =
                (self.playhead, self.viewport, self.last_view.as_ref())
        {
            let rate = last.rate;
            if self.follow == 2 {
                self.set_viewport(rate, position - (end - start) / 2, end - start);
            } else if self.follow == 1 && (position >= end || position < start) {
                self.set_viewport(rate, position - (end - start) / 20, end - start);
            }
        }
        let query = (self.viewport, columns);
        let refresh = self.last_query != Some(query)
            || self
                .last_refresh
                .is_none_or(|last| last.elapsed() >= Duration::from_millis(33));
        let fresh = refresh
            .then(|| {
                capture.view(
                    self.viewport.map(|range| range.0),
                    self.viewport.map(|range| range.1),
                    columns,
                )
            })
            .flatten();
        if fresh.is_some() {
            self.last_query = Some(query);
            self.last_refresh = Some(Instant::now());
        }
        // Capture performs disk work under its lock. Retain the last display instead of flashing empty.
        let view = fresh.or_else(|| self.last_view.take());
        let Some(view) = view else {
            self.empty(
                ui,
                "Your session appears here as you play",
                "Captured sections stay at their DAW timestamps across loops and seeks.",
            );
            return;
        };
        let height = (ui.available_height() - 220.0)
            .clamp(200.0, 390.0)
            .max(view.tracks.len() as f32 * 42.0 + 52.0);
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Input history");
                for action in Navigation::ALL {
                    let (title, hint, _) = action.control();
                    let available = !matches!(action, Navigation::Selection) || self.selection.is_some();
                    let available = available && (!matches!(action, Navigation::Center) || self.playhead.is_some());
                    if ui.add_enabled(available, egui::Button::new(title).small()).on_hover_text(hint).clicked() {
                        self.navigate(action, &view);
                    }
                }
                ui.label(egui::RichText::new(format!("{} – {}", time(view.start, view.rate), time(view.end, view.rate))).small().color(MUTED));
            });
            self.waveforms(ui, &view, height);
            self.navigator(ui, &view);
            ui.horizontal_wrapped(|ui| {
                for (mode, title) in [(0,"Follow off"), (1,"Catch up"), (2,"Lock playhead")] {
                    if ui.selectable_label(self.follow == mode, title).clicked() { self.follow = mode; }
                }
                ui.label(egui::RichText::new("Wheel zoom · Middle-drag pan · Alt-drag range").small().color(MUTED));
            });
            ui.horizontal_wrapped(|ui| {
                if ui.small_button("Select visible").on_hover_text("Select the displayed time range. Fit, zoom and navigation buttons also work with the keyboard.").clicked() {
                    self.selection = ordered(view.start, view.end);
                    self.viewport = Some((view.start, view.end));
                }
                if let Some((start, end)) = self.selection {
                    ui.label(egui::RichText::new(format!("Selection  {} – {}", time(start, view.rate), time(end, view.rate))).color(BLUE));
                    if ui.small_button("Clear selection").clicked() {
                        self.selection = None;
                    }
                } else {
                    ui.label(egui::RichText::new("Drag across a waveform to select an example.").color(MUTED));
                }
            });
            let can_label = self.selection.is_some() && !view.busy;
            ui.horizontal_wrapped(|ui| {
                ui.label("Label selection");
                if ui.add_enabled(can_label, egui::Button::new("Room")).clicked() {
                    self.apply_label(&capture, Some(0));
                }
                for track in &view.tracks {
                    if track.member < 8 && ui.add_enabled(can_label, egui::Button::new(format!("Mic {}", track.member + 1))).clicked() {
                        self.apply_label(&capture, Some(1 << track.member));
                    }
                }
                if ui.add_enabled(can_label, egui::Button::new("Erase label")).clicked() {
                    self.apply_label(&capture, None);
                }
            });
            if view.tracks.len() > 1 {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Overlap");
                    for track in &view.tracks {
                        if track.member >= 8 { continue; }
                        let bit = 1 << track.member;
                        let mut checked = self.overlap & bit != 0;
                        if ui.checkbox(&mut checked, format!("Mic {}", track.member + 1)).changed() {
                            if checked { self.overlap |= bit; } else { self.overlap &= !bit; }
                        }
                    }
                    if ui.add_enabled(can_label && self.overlap.count_ones() >= 2, egui::Button::new("Label overlap")).clicked() {
                        self.apply_label(&capture, Some(self.overlap));
                    }
                });
            }
            ui.add_space(5.0);
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(!view.busy && !view.labels.is_empty(), egui::Button::new(if view.busy { "Analyzing…" } else { "Analyze session" }).fill(BLUE.gamma_multiply(0.55))).clicked() {
                    self.error = capture.analyze(view.settings).err();
                }
                let mut enabled = params.enabled.value();
                if ui.add_enabled(view.ready || enabled, egui::Checkbox::new(&mut enabled, "Enable processing")).changed() {
                    set(setter, &params.enabled, enabled);
                }
                ui.label(egui::RichText::new(if view.busy { "Analysis in progress" } else if view.ready { "Analysis ready" } else { "Original audio preserved" }).color(if view.ready { TEAL } else { MUTED }));
            });
            ui.label(egui::RichText::new(&view.message).small().color(MUTED));
            if let Some(error) = &self.error {
                ui.label(egui::RichText::new(error).color(ERROR));
            }
            ui.collapsing(format!("Examples ({})", view.labels.len()), |ui| {
                if view.labels.is_empty() {
                    ui.label("Add room tone and solo speech for each microphone, then Analyze.");
                }
                egui::ScrollArea::vertical().id_salt("session-examples").max_height(110.0).show(ui, |ui| {
                    for label in &view.labels {
                        ui.push_id((label.start, label.end, label.mask), |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui.small_button("Show").clicked() {
                                    self.selection = Some((label.start, label.end));
                                    let padding = i64::from(view.rate);
                                    self.viewport = Some((label.start.saturating_sub(padding), label.end.saturating_add(padding)));
                                }
                                ui.label(format!("{}  {} – {}", label_name(label.mask), time(label.start, view.rate), time(label.end, view.rate)));
                                if ui.add_enabled(!view.busy, egui::Button::new("Remove").small()).clicked() {
                                    self.error = capture.label(label.start, label.end, None).err();
                                }
                            });
                        });
                    }
                });
            });
            ui.collapsing("Sound & cache", |ui| {
                ui.label("Sound changes take effect after Analyze.");
                let mut draft = view.settings;
                for (title, value, min, max, suffix) in [
                    ("Reduction", &mut draft.reduction_db, -48.0, 0.0, " dB"),
                    ("Fade in", &mut draft.attack_ms, 5.0, 80.0, " ms"),
                    ("Fade out", &mut draft.release_ms, 10.0, 150.0, " ms"),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(title);
                        ui.add(egui::Slider::new(value, min..=max).suffix(suffix));
                    });
                }
                if draft != view.settings {
                    self.error = capture.set_settings(draft).err();
                }
                ui.label(egui::RichText::new("Session sound is shared; host automation is not supported in this prototype.").small().color(MUTED));
                ui.label(egui::RichText::new("Analysis uses input channel 1; the same gain is applied to both stereo channels.").small().color(MUTED));
                ui.label(egui::RichText::new("Blank spans have no captured audio. Replayed audio verifies or updates the cache. Unverified regions preserve original audio.").small().color(MUTED));
                if !self.clear_armed {
                    if ui.button("Clear captured cache…").clicked() { self.clear_armed = true; }
                } else {
                    ui.label("Clear this group's captured audio, examples and analysis? Your DAW audio stays intact.");
                    ui.horizontal(|ui| {
                        if ui.add_enabled(!view.busy, egui::Button::new("Clear group cache")).clicked() {
                            self.error = capture.clear().err();
                            if self.error.is_none() {
                                self.viewport = None;
                                self.selection = None;
                                self.overlap = 0;
                                self.clear_armed = false;
                            }
                        }
                        if ui.button("Cancel").clicked() { self.clear_armed = false; }
                    });
                }
            });
        });
        if self.viewport != query.0 {
            ui.ctx().request_repaint();
        }
        self.last_view = Some(view);
    }

    fn empty(&self, ui: &mut egui::Ui, title: &str, detail: &str) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(
                ui.available_width(),
                ui.available_height().clamp(180.0, 350.0),
            ),
            Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, Color32::from_rgb(24, 27, 31));
        painter.text(
            rect.center() - egui::vec2(0.0, 18.0),
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(19.0),
            TEXT,
        );
        painter.text(
            rect.center() + egui::vec2(0.0, 17.0),
            Align2::CENTER_CENTER,
            detail,
            FontId::proportional(13.0),
            MUTED,
        );
    }

    fn apply_label(&mut self, capture: &crate::session_mode::SessionCapture, mask: Option<u8>) {
        if let Some((start, end)) = self.selection {
            self.error = capture.label(start, end, mask).err();
        }
    }

    fn zoom(&mut self, view: &SessionView, factor: f64) {
        let anchor = self
            .selection
            .map_or((view.start + view.end) / 2, |(a, b)| (a + b) / 2)
            .clamp(view.start, view.end);
        self.zoom_at(view, factor, anchor);
    }

    fn navigate(&mut self, action: Navigation, view: &SessionView) {
        match action {
            Navigation::Fit => self.viewport = None,
            Navigation::Selection => {
                if let Some((start, end)) = self.selection {
                    self.set_viewport(view.rate, start, end - start);
                }
            }
            Navigation::In => self.zoom(view, 0.8),
            Navigation::Out => self.zoom(view, 1.25),
            Navigation::Earlier => self.pan(view, -1),
            Navigation::Later => self.pan(view, 1),
            Navigation::Center => {
                if let Some(position) = self.playhead {
                    self.set_viewport(
                        view.rate,
                        position - (view.end - view.start) / 2,
                        view.end - view.start,
                    );
                }
            }
        }
        self.manual_navigation = Some(Instant::now());
    }

    fn navigator(&mut self, ui: &mut egui::Ui, view: &SessionView) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 18.0),
            Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, Color32::from_rgb(24, 27, 31));
        let total = view.captured_end.max(view.end).max(1);
        let left = rect.left() + rect.width() * view.start as f32 / total as f32;
        let right = rect.left() + rect.width() * view.end as f32 / total as f32;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(left, rect.top() + 3.0),
                Pos2::new(right.max(left + 4.0), rect.bottom() - 3.0),
            ),
            2.0,
            BORDER,
        );
        if (response.clicked() || response.dragged())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let center = sample_at(pointer.x, rect, 0, total);
            self.set_viewport(
                view.rate,
                center - (view.end - view.start) / 2,
                view.end - view.start,
            );
            self.manual_navigation = Some(Instant::now());
            ui.ctx().request_repaint();
        }
        response.on_hover_text("Captured timeline: click or drag to move the visible range.");
    }

    fn zoom_at(&mut self, view: &SessionView, factor: f64, anchor: i64) {
        let span = (view.end - view.start).max(1) as f64;
        let width = (span * factor).round().clamp(
            f64::from(view.rate.max(1)) / 10.0,
            f64::from(view.rate) * 7200.0,
        ) as i64;
        let fraction = (anchor - view.start) as f64 / span;
        self.set_viewport(
            view.rate,
            anchor - (fraction * width as f64).round() as i64,
            width,
        );
    }

    fn set_viewport(&mut self, rate: u32, start: i64, width: i64) {
        let total = i64::from(rate.max(1)) * 7200;
        let width = width.clamp((i64::from(rate) / 10).max(1), total);
        let start = start.clamp(0, total - width);
        self.viewport = Some((start, start + width));
    }

    fn pan(&mut self, view: &SessionView, direction: i64) {
        let shift = ((i128::from(view.end) - i128::from(view.start)) * i128::from(direction) * 4
            / 5)
        .clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        self.set_viewport(
            view.rate,
            view.start.saturating_add(shift),
            view.end - view.start,
        );
    }

    fn waveforms(&mut self, ui: &mut egui::Ui, view: &SessionView, height: f32) {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            Sense::click_and_drag(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, Color32::from_rgb(24, 27, 31));
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
        let plot = Rect::from_min_max(
            rect.min + egui::vec2(108.0, 27.0),
            rect.max - egui::vec2(12.0, 25.0),
        );
        if view.start >= view.end || view.tracks.is_empty() || plot.width() < 1.0 {
            return;
        }
        if response.hovered() {
            let (wheel, zoom, modifiers, pointer) = ui.input_mut(|input| {
                let wheel = input.smooth_scroll_delta;
                input.smooth_scroll_delta = Vec2::ZERO;
                (
                    wheel,
                    input.zoom_delta(),
                    input.modifiers,
                    input.pointer.hover_pos(),
                )
            });
            if let Some(pointer) = pointer {
                if wheel != Vec2::ZERO || (zoom - 1.0).abs() > f32::EPSILON {
                    self.manual_navigation = Some(Instant::now());
                }
                if (zoom - 1.0).abs() > f32::EPSILON {
                    self.zoom_at(
                        view,
                        f64::from(1.0 / zoom),
                        sample_at(pointer.x, plot, view.start, view.end),
                    );
                } else if modifiers.shift || wheel.x.abs() > wheel.y.abs() {
                    let pixels = if wheel.x != 0.0 { wheel.x } else { wheel.y };
                    let shift = (-f64::from(pixels) / f64::from(plot.width())
                        * (view.end - view.start) as f64)
                        .round() as i64;
                    self.set_viewport(
                        view.rate,
                        view.start.saturating_add(shift),
                        view.end - view.start,
                    );
                } else if wheel.y != 0.0 {
                    self.zoom_at(
                        view,
                        f64::from((-wheel.y * 0.004).exp()),
                        sample_at(pointer.x, plot, view.start, view.end),
                    );
                }
            }
            if response.double_clicked() {
                self.viewport = None;
            }
        }
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }
        if response.has_focus() {
            for action in Navigation::ALL {
                if ui
                    .input_mut(|input| input.consume_key(egui::Modifiers::NONE, action.control().2))
                {
                    self.navigate(action, view);
                }
            }
        }
        response.context_menu(|ui| {
            for action in Navigation::ALL {
                if ui.button(action.control().1).clicked() {
                    self.navigate(action, view);
                    ui.close();
                }
            }
            ui.separator();
            ui.label("Wheel: zoom at pointer");
            ui.label("Shift+wheel / middle-drag: pan");
            ui.label("Alt+drag: zoom range");
        });
        let span = (i128::from(view.end) - i128::from(view.start)) as f64;
        let x = |sample: i64| {
            plot.left()
                + (((i128::from(sample) - i128::from(view.start)) as f64 / span) as f32
                    * plot.width())
        };
        for step in 0..=5 {
            let xx = plot.left() + plot.width() * step as f32 / 5.0;
            painter.line_segment(
                [Pos2::new(xx, plot.top()), Pos2::new(xx, plot.bottom())],
                Stroke::new(1.0, Color32::from_rgb(42, 47, 54)),
            );
            let sample = (i128::from(view.start)
                + (i128::from(view.end) - i128::from(view.start)) * step / 5)
                as i64;
            painter.text(
                Pos2::new(xx, rect.bottom() - 11.0),
                if step == 0 {
                    Align2::LEFT_CENTER
                } else if step == 5 {
                    Align2::RIGHT_CENTER
                } else {
                    Align2::CENTER_CENTER
                },
                time(sample, view.rate),
                FontId::monospace(11.0),
                MUTED,
            );
        }
        let lane_height = plot.height() / view.tracks.len() as f32;
        for (index, track) in view.tracks.iter().enumerate() {
            let center = plot.top() + lane_height * (index as f32 + 0.5);
            let color = if track.verified {
                if index % 2 == 0 { TEAL } else { BLUE }
            } else {
                MUTED.gamma_multiply(0.6)
            };
            painter.text(
                Pos2::new(rect.left() + 13.0, center - 7.0),
                Align2::LEFT_CENTER,
                format!("Mic {}", track.member + 1),
                FontId::proportional(13.0),
                TEXT,
            );
            painter.text(
                Pos2::new(rect.left() + 13.0, center + 10.0),
                Align2::LEFT_CENTER,
                if !track.present {
                    "Not connected"
                } else if track.verified {
                    "Captured input"
                } else {
                    "Replay to verify"
                },
                FontId::proportional(10.0),
                MUTED,
            );
            painter.line_segment(
                [
                    Pos2::new(plot.left(), center),
                    Pos2::new(plot.right(), center),
                ],
                Stroke::new(1.0, BORDER),
            );
            let scale = track.peaks.iter().fold(0.03_f32, |scale, peak| {
                scale.max(peak.min.abs()).max(peak.max.abs())
            });
            let mut mesh = egui::Mesh::default();
            for peak in &track.peaks {
                if peak.end <= view.start
                    || peak.start >= view.end
                    || !peak.min.is_finite()
                    || !peak.max.is_finite()
                {
                    continue;
                }
                let left = x(peak.start.max(view.start)).max(plot.left());
                let right = x(peak.end.min(view.end)).min(plot.right()).max(left + 0.5);
                let top = center - peak.max / scale * lane_height * 0.38;
                let bottom = (center - peak.min / scale * lane_height * 0.38).max(top + 1.0);
                mesh.add_colored_rect(
                    Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom)),
                    color,
                );
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        for label in &view.labels {
            if label.end <= view.start || label.start >= view.end {
                continue;
            }
            let label_rect = Rect::from_min_max(
                Pos2::new(x(label.start.max(view.start)), rect.top() + 5.0),
                Pos2::new(x(label.end.min(view.end)), plot.top() - 4.0),
            );
            painter.rect_filled(label_rect, 2.0, BLUE.gamma_multiply(0.45));
            if label_rect.width() > 35.0 {
                painter.with_clip_rect(label_rect).text(
                    label_rect.center(),
                    Align2::CENTER_CENTER,
                    label_name(label.mask),
                    FontId::proportional(10.0),
                    TEXT,
                );
            }
        }
        if let Some(position) = self.playhead
            && position >= view.start
            && position <= view.end
        {
            painter.line_segment(
                [
                    Pos2::new(x(position), plot.top()),
                    Pos2::new(x(position), plot.bottom()),
                ],
                Stroke::new(1.5, TEXT),
            );
        }
        // A complete gesture may arrive between redraws; button_down/drag_started_by then miss it.
        let (primary_press, middle_press, primary_release) = ui.input(|input| {
            let mut primary = None;
            let mut middle = None;
            let mut released = None;
            for event in &input.events {
                if let egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                } = event
                {
                    match (*button, *pressed) {
                        (egui::PointerButton::Primary, true) => primary = Some((*pos, *modifiers)),
                        (egui::PointerButton::Middle, true) => middle = Some(*pos),
                        (egui::PointerButton::Primary, false) => released = Some(*pos),
                        _ => {}
                    }
                }
            }
            (primary, middle, released)
        });
        if response.contains_pointer()
            && let Some(origin) = middle_press
            && plot.contains(origin)
        {
            self.pan_anchor = Some((origin.x, view.start, view.end));
        }
        if let Some((origin, start, end)) = self.pan_anchor {
            self.manual_navigation = Some(Instant::now());
            if let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) {
                let shift = (f64::from(origin - pointer.x) / f64::from(plot.width())
                    * (end - start) as f64)
                    .round() as i64;
                self.set_viewport(view.rate, start.saturating_add(shift), end - start);
            }
            if !ui.input(|input| input.pointer.button_down(egui::PointerButton::Middle)) {
                self.pan_anchor = None;
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        if response.contains_pointer()
            && let Some((origin, modifiers)) = primary_press
            && plot.contains(origin)
        {
            self.drag_anchor = Some(sample_at(origin.x, plot, view.start, view.end));
            self.viewport = Some((view.start, view.end));
            self.range_zoom = modifiers.alt;
            response.request_focus();
        }
        if (response.dragged() || response.drag_stopped() || primary_release.is_some())
            && let (Some(anchor), Some(pointer)) = (
                self.drag_anchor,
                primary_release.or_else(|| ui.input(|input| input.pointer.interact_pos())),
            )
        {
            let range = ordered(anchor, sample_at(pointer.x, plot, view.start, view.end));
            if self.range_zoom {
                self.zoom_range = range;
            } else {
                self.selection = range;
            }
        }
        if primary_release.is_some() && self.drag_anchor.is_some() {
            if self.range_zoom
                && let Some((start, end)) = self.zoom_range.take()
            {
                self.set_viewport(view.rate, start, end - start);
            }
            self.drag_anchor = None;
            self.range_zoom = false;
            self.zoom_range = None;
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.selection = None;
            self.drag_anchor = None;
            self.pan_anchor = None;
            self.range_zoom = false;
            self.zoom_range = None;
        }
        if let Some((start, end)) = self.zoom_range.or(self.selection)
            && end > view.start
            && start < view.end
        {
            let selected = Rect::from_min_max(
                Pos2::new(x(start.max(view.start)), plot.top()),
                Pos2::new(x(end.min(view.end)), plot.bottom()),
            );
            painter.rect_filled(selected, 0.0, BLUE.linear_multiply(0.18));
            painter.rect_stroke(selected, 0.0, Stroke::new(1.0, BLUE), StrokeKind::Inside);
        }
        response.on_hover_text("Wheel: zoom at pointer · Shift+wheel: pan · Middle-drag: pan · Alt+drag: zoom range · Double-click: fit history · Drag: select · Esc: clear");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn navigation_view() -> SessionView {
        SessionView {
            rate: 48000,
            start: 480000,
            end: 3360000,
            captured_end: 3360000,
            tracks: vec![crate::session_mode::TrackView {
                member: 0,
                present: true,
                verified: true,
                peaks: vec![],
            }],
            labels: vec![],
            message: String::new(),
            busy: false,
            ready: false,
            settings: crate::session_analysis::Settings::default(),
        }
    }

    fn navigation_frame(
        context: &egui::Context,
        workspace: &mut SessionUi,
        view: &SessionView,
        mut events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> Rect {
        events.insert(0, egui::Event::ModifiersChanged(modifiers));
        let mut plot = Rect::NOTHING;
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 440.0))),
                events,
                ..Default::default()
            },
            |ui| {
                let rect = Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(ui.available_width(), 390.0),
                );
                plot = Rect::from_min_max(
                    rect.min + egui::vec2(108.0, 27.0),
                    rect.max - egui::vec2(12.0, 25.0),
                );
                workspace.waveforms(ui, view, 390.0);
            },
        );
        output.textures_delta.clear();
        plot
    }

    #[test]
    fn wheel_zooms_at_pointer_without_changing_selection() {
        let context = egui::Context::default();
        let mut workspace = SessionUi {
            selection: Some((800000, 900000)),
            ..Default::default()
        };
        let view = navigation_view();
        let plot = navigation_frame(
            &context,
            &mut workspace,
            &view,
            vec![egui::Event::PointerMoved(Pos2::new(330.0, 200.0))],
            egui::Modifiers::NONE,
        );
        navigation_frame(
            &context,
            &mut workspace,
            &view,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 120.0),
                modifiers: egui::Modifiers::NONE,
                source: egui::MouseWheelSource::Unknown,
                phase: egui::TouchPhase::Move,
            }],
            egui::Modifiers::NONE,
        );
        let (start, end) = workspace.viewport.expect("wheel must zoom the waveform");
        assert!(end - start < view.end - view.start);
        let fraction = f64::from((330.0 - plot.left()) / plot.width());
        let before = view.start as f64 + fraction * (view.end - view.start) as f64;
        let after = start as f64 + fraction * (end - start) as f64;
        assert!(
            (before - after).abs() <= 2.0,
            "pointer time shifted: {before} / {after}"
        );
        assert_eq!(workspace.selection, Some((800000, 900000)));
    }

    #[test]
    fn zoom_and_pan_keep_the_span_at_timeline_boundaries() {
        let mut workspace = SessionUi::default();
        workspace.set_viewport(48000, -48000, 96000);
        assert_eq!(workspace.viewport, Some((0, 96000)));
        workspace.set_viewport(48000, 48000 * 7200, 96000);
        assert_eq!(
            workspace.viewport,
            Some((48000 * 7200 - 96000, 48000 * 7200))
        );
        let view = navigation_view();
        for _ in 0..30 {
            workspace.zoom_at(&view, 1e30, view.start);
            let (start, end) = workspace.viewport.unwrap();
            assert_eq!((start, end), (0, 48000 * 7200));
        }
    }

    #[test]
    fn middle_pan_and_alt_range_zoom_preserve_existing_example_selection() {
        for button in [egui::PointerButton::Middle, egui::PointerButton::Primary] {
            let context = egui::Context::default();
            let mut workspace = SessionUi {
                selection: Some((800000, 900000)),
                ..Default::default()
            };
            let view = navigation_view();
            let modifiers = if button == egui::PointerButton::Primary {
                egui::Modifiers::ALT
            } else {
                egui::Modifiers::NONE
            };
            navigation_frame(
                &context,
                &mut workspace,
                &view,
                vec![egui::Event::PointerMoved(Pos2::new(330.0, 200.0))],
                modifiers,
            );
            navigation_frame(
                &context,
                &mut workspace,
                &view,
                vec![
                    egui::Event::PointerMoved(Pos2::new(330.0, 200.0)),
                    egui::Event::PointerButton {
                        pos: Pos2::new(330.0, 200.0),
                        button,
                        pressed: true,
                        modifiers,
                    },
                ],
                modifiers,
            );
            navigation_frame(
                &context,
                &mut workspace,
                &view,
                vec![egui::Event::PointerMoved(Pos2::new(530.0, 200.0))],
                modifiers,
            );
            navigation_frame(
                &context,
                &mut workspace,
                &view,
                vec![egui::Event::PointerButton {
                    pos: Pos2::new(530.0, 200.0),
                    button,
                    pressed: false,
                    modifiers,
                }],
                modifiers,
            );
            let (start, end) = workspace
                .viewport
                .unwrap_or_else(|| panic!("{button:?} gesture must navigate"));
            assert_eq!(
                workspace.selection,
                Some((800000, 900000)),
                "{button:?} altered selection"
            );
            if button == egui::PointerButton::Middle {
                assert!(start < view.start);
                assert_eq!(end - start, view.end - view.start);
            } else {
                assert!(start > view.start);
                assert!(end - start < view.end - view.start);
            }
        }
    }

    #[test]
    fn fast_drag_released_between_redraws_still_selects_source_time() {
        let context = egui::Context::default();
        let mut workspace = SessionUi::default();
        let view = navigation_view();
        navigation_frame(
            &context,
            &mut workspace,
            &view,
            vec![egui::Event::PointerMoved(Pos2::new(330.0, 200.0))],
            egui::Modifiers::NONE,
        );
        navigation_frame(
            &context,
            &mut workspace,
            &view,
            vec![
                egui::Event::PointerButton {
                    pos: Pos2::new(330.0, 200.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerMoved(Pos2::new(530.0, 200.0)),
                egui::Event::PointerButton {
                    pos: Pos2::new(530.0, 200.0),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            egui::Modifiers::NONE,
        );
        assert!(
            workspace.selection.is_some(),
            "a drag completed between redraws must not disappear"
        );
        navigation_frame(
            &context,
            &mut workspace,
            &view,
            vec![egui::Event::Key {
                key: egui::Key::F,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            egui::Modifiers::NONE,
        );
        assert_eq!(
            workspace.viewport, workspace.selection,
            "focused F uses the same Fit selection action as the toolbar"
        );
    }

    #[test]
    fn missing_capture_shows_restart_and_binding_errors_before_setup_advice() {
        let probe = crate::FlProbe::default();
        let state = GuiState::new();
        for (status, error, expected) in [
            (
                RuntimeStatus::Restarting,
                None,
                "Waiting for the host to reconnect",
            ),
            (
                RuntimeStatus::Disabled,
                Some("This Session cache is open in another process"),
                "Cannot connect this Session",
            ),
        ] {
            state
                .status
                .store(status as u8, std::sync::atomic::Ordering::Relaxed);
            *state.error.lock() = error.map(str::to_owned);
            let context = egui::Context::default();
            let mut workspace = SessionUi::default();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 680.0))),
                    ..Default::default()
                },
                |ui| workspace.show(ui, &probe.params, None, &state),
            );
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,egui::Shape::Text(text) if text.galley.job.text == expected)),"missing {expected}");
            output.textures_delta.clear();
        }
    }

    #[test]
    fn waveform_render_work_measurement() {
        let context = egui::Context::default();
        let mut workspace = SessionUi::default();
        let mut view = navigation_view();
        view.tracks = (0..8)
            .map(|member| crate::session_mode::TrackView {
                member,
                present: true,
                verified: true,
                peaks: (0..1000)
                    .map(|column| crate::session_mode::WavePeak {
                        start: view.start + (view.end - view.start) * column / 1000,
                        end: view.start + (view.end - view.start) * (column + 1) / 1000,
                        min: -0.5,
                        max: 0.5,
                    })
                    .collect(),
            })
            .collect();
        let begin = std::time::Instant::now();
        let mut count = 0;
        for _ in 0..40 {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1200.0, 440.0))),
                    ..Default::default()
                },
                |ui| workspace.waveforms(ui, &view, 390.0),
            );
            count = output.shapes.len();
            std::hint::black_box(
                context.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point),
            );
            output.textures_delta.clear();
        }
        eprintln!(
            "waveform CPU draw+tessellate: {:.3} ms/frame; {count} shapes (8 tracks, 1000 columns, 40 frames)",
            begin.elapsed().as_secs_f64() * 25.0
        );
    }

    #[test]
    fn selection_maps_to_source_time_and_preserves_reverse_drags() {
        let rect = Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(500.0, 200.0));
        assert_eq!(sample_at(100.0, rect, 480000, 1440000), 480000);
        assert_eq!(sample_at(300.0, rect, 480000, 1440000), 960000);
        assert_eq!(sample_at(900.0, rect, 480000, 1440000), 1440000);
        assert_eq!(sample_at(-100.0, rect, -48000, 0), -48000);
        assert_eq!(ordered(50, 10), Some((10, 50)));
        assert_eq!(ordered(10, 10), None);
        assert_eq!(label_name(0), "Room");
        assert_eq!(label_name(0b10000001), "Mic 1 + Mic 8");
    }
    #[test]
    fn full_session_workspace_renders_actual_shared_capture_and_controls() {
        use crate::session_mode::integration_tests::{Fixture, direct_capture};
        let fixture = Fixture::new(8);
        for member in 0..8 {
            direct_capture(fixture.engine(), member, 0, 8000 * 8, false);
        }
        fixture.engine().label(0, 8000, Some(0)).unwrap();
        fixture.engine().label(16000, 24000, Some(129)).unwrap();
        let state = GuiState::new();
        *state.capture.lock() = Some(fixture.engine().clone());
        let mut probe = crate::FlProbe::default();
        Arc::get_mut(&mut probe.params).unwrap().group =
            IntParam::new("Session group", 1, IntRange::Linear { min: 0, max: 8 });
        let params = probe.params.clone();
        for (name, width, height) in [
            ("session-workspace", 1000, 680),
            ("session-workspace-compact", 720, 560),
        ] {
            let context = egui::Context::default();
            crate::editor::style(&context);
            let mut workspace = SessionUi::default();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        Pos2::ZERO,
                        egui::vec2(width as f32, height as f32),
                    )),
                    ..Default::default()
                },
                |ui| {
                    workspace.show(ui, &params, None, &state);
                    assert!(ui.min_rect().right() <= width as f32 + 1.0);
                },
            );
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
    fn capture_worker_contention_retains_the_last_waveform_and_selection() {
        use crate::session_mode::integration_tests::{Fixture, direct_capture};
        let fixture = Fixture::new(2);
        for member in 0..2 {
            direct_capture(fixture.engine(), member, 0, 8000, false);
        }
        let state = GuiState::new();
        *state.capture.lock() = Some(fixture.engine().clone());
        let probe = crate::FlProbe::default();
        let context = egui::Context::default();
        let mut workspace = SessionUi::default();
        let frame = |workspace: &mut SessionUi| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 680.0))),
                    ..Default::default()
                },
                |ui| workspace.show(ui, &probe.params, None, &state),
            );
            output.textures_delta.clear();
        };
        frame(&mut workspace);
        assert_eq!(workspace.last_view.as_ref().unwrap().tracks.len(), 2);
        workspace.selection = Some((100, 500));
        workspace.last_refresh = None;
        fixture.with_worker_lock(|| frame(&mut workspace));
        assert_eq!(workspace.last_view.as_ref().unwrap().tracks.len(), 2);
        assert_eq!(workspace.selection, Some((100, 500)));
    }

    #[test]
    fn eight_source_time_lanes_render_at_compact_and_full_widths() {
        use crate::{
            session_analysis::{LabelRange, Settings},
            session_mode::{TrackView, WavePeak},
        };
        let view = SessionView {
            rate: 48000,
            start: 480000,
            end: 3360000,
            captured_end: 3360000,
            tracks: (0..8)
                .map(|member| TrackView {
                    member,
                    present: member != 7,
                    verified: member != 6,
                    peaks: (0..300)
                        .filter(|index| !(100..150).contains(index))
                        .map(|index| {
                            let amplitude =
                                0.05 + ((index * 37 + member * 11) % 100) as f32 / 120.0;
                            WavePeak {
                                start: 480000 + index as i64 * 9600,
                                end: 480000 + (index as i64 + 1) * 9600,
                                min: -amplitude,
                                max: amplitude * 0.8,
                            }
                        })
                        .collect(),
                })
                .collect(),
            labels: vec![
                LabelRange {
                    start: 600000,
                    end: 900000,
                    mask: 0,
                },
                LabelRange {
                    start: 2000000,
                    end: 2400000,
                    mask: 129,
                },
            ],
            message: String::new(),
            busy: false,
            ready: false,
            settings: Settings::default(),
        };
        for (name, width) in [
            ("session-timeline", 1000),
            ("session-timeline-compact", 720),
        ] {
            let context = egui::Context::default();
            context.set_visuals(egui::Visuals::dark());
            let mut workspace = SessionUi {
                selection: Some((2500000, 2800000)),
                ..Default::default()
            };
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        Pos2::ZERO,
                        egui::vec2(width as f32, 410.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    workspace.waveforms(ui, &view, 390.0);
                    assert!(ui.min_rect().right() <= width as f32 + 1.0);
                    assert!(ui.min_rect().bottom() <= 410.0);
                },
            );
            assert!(!output.shapes.is_empty());
            if let Ok(directory) = std::env::var("CLEANSLATE_PLUGIN_CAPTURES") {
                std::fs::create_dir_all(&directory).unwrap();
                crate::editor_capture::write_capture(
                    &context,
                    &mut output,
                    &std::path::Path::new(&directory).join(format!("{name}.ppm")),
                    width,
                    410,
                )
                .unwrap();
            } else {
                output.textures_delta.clear();
            }
        }
    }
}
