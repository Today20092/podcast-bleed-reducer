use crate::{
    CONSENT_MASK, Consent, DetectionState, GateParams,
    history::{BINS, Bin},
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke};
use nice_plug::context::gui::GuiContext;
use nice_plug_egui::NiceEguiApp;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
const INPUT: Color32 = Color32::from_rgb(125, 140, 155);
const OUTPUT: Color32 = Color32::from_rgb(78, 215, 190);
const REDUCTION: Color32 = Color32::from_rgb(191, 164, 244);
const MUTED: Color32 = Color32::from_rgb(173, 185, 198);
pub struct Panel {
    params: Arc<GateParams>,
    consent: Arc<AtomicU64>,
    detection_state: Arc<AtomicU8>,
    context: Option<GuiContext>,
    last_meter: u64,
    last_update: f64,
    generation: u64,
    bins: [Option<Bin>; BINS],
    frozen: bool,
    show_original: bool,
    show_processed: bool,
}
impl Panel {
    pub fn new(
        params: Arc<GateParams>,
        consent: Arc<AtomicU64>,
        detection_state: Arc<AtomicU8>,
    ) -> Self {
        Self {
            params,
            consent,
            detection_state,
            context: None,
            last_meter: 0,
            last_update: f64::NEG_INFINITY,
            generation: 0,
            bins: [None; BINS],
            frozen: false,
            show_original: true,
            show_processed: true,
        }
    }
}
impl NiceEguiApp for Panel {
    fn build(
        &mut self,
        context: egui::Context,
        gui: GuiContext,
        _: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        style(&context);
        self.context = Some(gui);
        Ok(())
    }
    fn editor_closed(&mut self) {
        self.context = None;
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut nice_plug_egui::Frame) {
        self.show(ui);
    }
}
pub(crate) fn style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(32, 34, 38);
    visuals.override_text_color = Some(Color32::from_rgb(228, 231, 235));
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(52, 57, 64);
    visuals.selection.stroke = Stroke::new(2.0, Color32::from_rgb(154, 197, 255));
    context.set_visuals(visuals);
}
fn reduction(gain: f32) -> f32 {
    if gain.is_finite() && gain > 0.0 {
        (-20.0 * gain.log10()).clamp(0.0, 6.0)
    } else {
        0.0
    }
}
fn db(peak: f32) -> f32 {
    if peak > 0.0 {
        20.0 * peak.log10()
    } else {
        -120.0
    }
}
impl Panel {
    fn graph(&self, ui: &mut egui::Ui, current: Option<Bin>, fresh: bool, applied: f32) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(
                ui.available_width(),
                (ui.available_height() - 34.0).max(224.0),
            ),
            egui::Sense::hover(),
        );
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 4.0, Color32::from_rgb(18, 21, 27));
        let left = rect.left() + 14.0;
        let right = rect.right() - 120.0;
        let width = right - left;
        let bottom = rect.bottom() - 27.0;
        let gain_top = rect.top() + 25.0;
        let gain_height = 46.0;
        let wave_top = gain_top + gain_height + 26.0;
        let wave_height = bottom - wave_top;
        let text = |position, align, label: &str, color| {
            p.text(position, align, label, FontId::proportional(12.0), color);
        };
        text(
            Pos2::new(left, rect.top() + 7.0),
            Align2::LEFT_TOP,
            "Applied reduction",
            REDUCTION,
        );
        text(
            Pos2::new(right, gain_top),
            Align2::RIGHT_BOTTOM,
            "0 dB",
            REDUCTION,
        );
        text(
            Pos2::new(right, gain_top + gain_height),
            Align2::RIGHT_TOP,
            "−6 dB",
            REDUCTION,
        );
        for second in 0..=5 {
            let x = right - width * second as f32 / 5.12;
            p.line_segment(
                [Pos2::new(x, gain_top), Pos2::new(x, bottom)],
                Stroke::new(1.0, Color32::from_rgb(36, 41, 50)),
            );
            let label = if second == 0 {
                "Now".into()
            } else {
                format!("−{second}s")
            };
            text(
                Pos2::new(x, rect.bottom() - 5.0),
                Align2::CENTER_BOTTOM,
                &label,
                MUTED,
            );
        }
        for level in [0, -18, -36, -54, -72, -90] {
            let y = bottom - (level as f32 + 90.0) / 90.0 * wave_height;
            p.line_segment(
                [Pos2::new(left, y), Pos2::new(right, y)],
                Stroke::new(1.0, Color32::from_rgb(40, 45, 54)),
            );
            text(
                Pos2::new(right - 4.0, y - 2.0),
                Align2::RIGHT_BOTTOM,
                &format!("{level}"),
                MUTED,
            );
        }
        let step = width / BINS as f32;
        for (visible, input, color) in [
            (self.show_original, true, INPUT),
            (self.show_processed, false, OUTPUT),
        ] {
            if !visible {
                continue;
            }
            let mut mesh = egui::epaint::Mesh::default();
            let mut previous: Option<Pos2> = None;
            for (i, bin) in self.bins.iter().enumerate() {
                let Some(bin) = bin else {
                    previous = None;
                    continue;
                };
                let peak = if input { bin.input } else { bin.output };
                let y = bottom - ((db(peak) + 90.0) / 90.0).clamp(0.0, 1.0) * wave_height;
                let point = Pos2::new(left + i as f32 * step, y);
                if let Some(old) = previous {
                    let base = mesh.vertices.len() as u32;
                    let top_color = Color32::from_rgba_unmultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        if input { 150 } else { 175 },
                    );
                    let base_color =
                        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 24);
                    mesh.colored_vertex(old, top_color);
                    mesh.colored_vertex(point, top_color);
                    mesh.colored_vertex(Pos2::new(point.x, bottom), base_color);
                    mesh.colored_vertex(Pos2::new(old.x, bottom), base_color);
                    mesh.add_triangle(base, base + 1, base + 2);
                    mesh.add_triangle(base, base + 2, base + 3);
                    p.line_segment([old, point], Stroke::new(1.0, color));
                }
                previous = Some(point);
            }
            p.add(egui::Shape::mesh(mesh));
        }
        let mut previous = None;
        let mut peak_reduction = (0.0_f32, Pos2::ZERO);
        let mut gain_mesh = egui::epaint::Mesh::default();
        for (i, bin) in self.bins.iter().enumerate() {
            let Some(bin) = bin else {
                previous = None;
                continue;
            };
            let value = reduction(bin.gain);
            let point = Pos2::new(left + i as f32 * step, gain_top + value / 6.0 * gain_height);
            if value > peak_reduction.0 {
                peak_reduction = (value, point);
            }
            if let Some(old) = previous {
                p.line_segment([old, point], Stroke::new(1.7, REDUCTION));
                let base = gain_mesh.vertices.len() as u32;
                let shade = Color32::from_rgba_unmultiplied(191, 164, 244, 18);
                for point in [
                    Pos2::new(old.x, gain_top),
                    Pos2::new(point.x, gain_top),
                    point,
                    old,
                ] {
                    gain_mesh.colored_vertex(point, shade);
                }
                gain_mesh.add_triangle(base, base + 1, base + 2);
                gain_mesh.add_triangle(base, base + 2, base + 3);
            }
            previous = Some(point);
        }
        p.add(egui::Shape::mesh(gain_mesh));
        if peak_reduction.0 >= 0.1 {
            let position = Pos2::new(
                peak_reduction.1.x.clamp(left + 35.0, right - 35.0),
                gain_top + gain_height + 8.0,
            );
            text(
                position,
                Align2::CENTER_TOP,
                &format!("−{:.1} dB peak", peak_reduction.0),
                REDUCTION,
            );
        }
        text(
            Pos2::new(left + 4.0, wave_top + 8.0),
            Align2::LEFT_TOP,
            "Original",
            INPUT,
        );
        text(
            Pos2::new(left + 74.0, wave_top + 8.0),
            Align2::LEFT_TOP,
            "Processed",
            OUTPUT,
        );
        if self.detection_state.load(Ordering::Acquire) == DetectionState::Failed as u8 {
            text(
                Pos2::new((left + right) / 2.0, wave_top + wave_height / 2.0),
                Align2::CENTER_CENTER,
                "Gate unavailable · original audio is preserved",
                Color32::from_rgb(249, 151, 125),
            );
        } else if self.bins.iter().all(Option::is_none) {
            text(
                Pos2::new((left + right) / 2.0, wave_top + wave_height / 2.0),
                Align2::CENTER_CENTER,
                "Play audio in your host to see the signal",
                MUTED,
            );
        }
        for (i, name, peak, color) in [
            (0, "IN", current.map_or(0.0, |b| b.input), INPUT),
            (1, "OUT", current.map_or(0.0, |b| b.output), OUTPUT),
            (2, "GR", applied, REDUCTION),
        ] {
            let x = rect.right() - 98.0 + i as f32 * 34.0;
            text(
                Pos2::new(x + 7.0, rect.top() + 8.0),
                Align2::CENTER_TOP,
                name,
                MUTED,
            );
            let bar =
                Rect::from_min_max(Pos2::new(x, gain_top), Pos2::new(x + 14.0, bottom - 28.0));
            p.rect_filled(bar, 2.0, Color32::from_rgb(41, 47, 55));
            let level = if i == 2 {
                applied / 6.0
            } else if fresh {
                ((db(peak) + 90.0) / 90.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            p.rect_filled(
                Rect::from_min_max(
                    Pos2::new(bar.left(), bar.bottom() - bar.height() * level),
                    bar.max,
                ),
                2.0,
                if i != 2 && peak > 1.0 && fresh {
                    Color32::from_rgb(249, 129, 113)
                } else {
                    color
                },
            );
            let label = if i == 2 {
                format!("{applied:.1}")
            } else if fresh && peak > 0.000001 {
                format!("{:.0}", db(peak))
            } else {
                "−∞".into()
            };
            text(
                Pos2::new(x + 7.0, bottom - 17.0),
                Align2::CENTER_TOP,
                &label,
                MUTED,
            );
        }
        text(
            Pos2::new(rect.right() - 74.0, rect.bottom() - 5.0),
            Align2::CENTER_BOTTOM,
            "dBFS",
            MUTED,
        );
        text(
            Pos2::new(rect.right() - 23.0, rect.bottom() - 5.0),
            Align2::CENTER_BOTTOM,
            "dB",
            MUTED,
        );
        let pod = Rect::from_min_size(
            Pos2::new(left + 12.0, bottom - 57.0),
            egui::vec2(265.0, 43.0),
        );
        p.rect_filled(pod, 5.0, Color32::from_rgb(37, 42, 50));
        let mut range = self.params.range.value();
        let response = ui.put(
            pod.shrink(10.0),
            egui::Slider::new(&mut range, 0.0..=6.0)
                .text("Range")
                .suffix(" dB"),
        );
        if response.changed()
            && let Some(context) = &self.context
        {
            let setter = context.param_setter();
            setter.begin_set_parameter(&self.params.range);
            setter.set_parameter(&self.params.range, range);
            setter.end_set_parameter(&self.params.range);
        }
        response.on_hover_text(
            "Maximum reduction, 0–6 dB. Drag the value or click it to type an exact number.",
        );
        ui.advance_cursor_after_rect(rect);
    }
    pub(crate) fn status(&self) -> &'static str {
        let state = self.consent.load(Ordering::Acquire) & CONSENT_MASK;
        if state == Consent::Unavailable as u64 {
            "Not prepared · original audio"
        } else if self.detection_state.load(Ordering::Acquire) == DetectionState::Failed as u8 {
            "Processing unavailable · original audio"
        } else if self.params.bypass.value() {
            "Bypassed · original audio"
        } else if state == Consent::Armed as u64 {
            if self.detection_state.load(Ordering::Acquire) == DetectionState::Current as u8 {
                "Last block · current detection data"
            } else {
                "Waiting for current detection data · original audio"
            }
        } else {
            "Not armed · original audio"
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        style(ui.ctx());
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(32, 34, 38));
        let now = ui.input(|input| input.time);
        let packet = self.params.meter.load(Ordering::Relaxed);
        if packet != self.last_meter {
            self.last_meter = packet;
            self.last_update = now;
        }
        let fresh = packet != 0 && now - self.last_update < 0.25;
        let enabled = !self.params.bypass.value()
            && self.consent.load(Ordering::Acquire) & CONSENT_MASK == Consent::Armed as u64;
        let applied = if enabled && fresh {
            reduction(f32::from_bits(packet as u32))
        } else {
            0.0
        };
        let generation = self.params.history.generation.load(Ordering::SeqCst);
        if generation != self.generation {
            self.generation = generation;
            self.bins.fill(None);
            self.frozen = false;
        }
        let latest = self.params.history.latest.load(Ordering::SeqCst);
        if !self.frozen {
            for (i, bin) in self.bins.iter_mut().enumerate() {
                *bin = self
                    .params
                    .history
                    .read(latest.saturating_sub((BINS - 1 - i) as u64));
            }
        }
        let current = self.params.history.read(latest);
        egui::Frame::NONE.inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);
            self.graph(ui, current, fresh, applied);
            ui.horizontal(|ui| {
                let mut bypass = self.params.bypass.value();
                let response = ui.checkbox(&mut bypass, "Bypass");
                if response.changed() && let Some(context) = &self.context {
                    let setter = context.param_setter();
                    setter.begin_set_parameter(&self.params.bypass);
                    setter.set_parameter(&self.params.bypass, bypass);
                    setter.end_set_parameter(&self.params.bypass);
                }
                response.on_hover_text("Original audio. Enabling the gate does not turn Bypass off.");
                let consent = self.consent.load(Ordering::Acquire);
                let armed = consent & CONSENT_MASK == Consent::Armed as u64;
                let action = if armed { "Disable gate" } else { "Enable gate" };
                let response = ui.push_id(consent & !CONSENT_MASK, |ui| {
                    ui.add_enabled(consent & CONSENT_MASK != Consent::Unavailable as u64,
                        egui::Button::new(action).fill(if armed { Color32::from_rgb(38, 80, 72) } else { Color32::from_rgb(52, 57, 64) }))
                }).inner;
                if response.clicked() {
                    let next = if armed { Consent::Ready } else { Consent::Armed };
                    let _ = self.consent.compare_exchange(consent, (consent & !CONSENT_MASK) | next as u64, Ordering::AcqRel, Ordering::Acquire);
                }
                response.on_hover_text(format!("{}\nExperimental speech gating, up to the selected Range. Consent is temporary and is never restored from a project.\nFixed timing: 300 ms hold, 5 ms opening, 250 ms closing; evidence expires after 100 ms.", self.status()))
                    .on_disabled_hover_text("Gate unavailable. Activate the plugin at 44.1, 48 or 96 kHz; original audio is preserved.");
                ui.checkbox(&mut self.frozen, "Freeze").on_hover_text("Freeze history only. Live meters and audio processing continue.");
                ui.checkbox(&mut self.show_original, "Original");
                ui.checkbox(&mut self.show_processed, "Processed");
            });
        });
        let interval = if fresh && !self.frozen {
            16
        } else if fresh {
            33
        } else {
            100
        };
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(interval));
    }
}
