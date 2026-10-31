use crate::{
    CONSENT_MASK, Consent, DetectionState, GateParams,
    history::{BINS, Bin},
};
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke};
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
    fn graph(&self, ui: &mut egui::Ui, current: Option<Bin>, fresh: bool) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(
                ui.available_width(),
                (ui.available_height() - 150.0).max(200.0),
            ),
            egui::Sense::hover(),
        );
        response.on_hover_text("5.12 seconds of real audio. Waveforms show 10 ms peak envelopes, combined across stereo channels. Purple shows maximum applied reduction in each matching interval. Freeze affects display only.");
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 4.0, Color32::from_rgb(17, 21, 26));
        let left = rect.left() + 10.0;
        let right = rect.right() - 84.0;
        let width = right - left;
        let center = rect.top() + rect.height() * 0.35;
        let height = rect.height() * 0.23;
        let gain_top = rect.top() + rect.height() * 0.685;
        let gain_height = rect.height() * 0.17;
        let text = |position, align, label: &str, color| {
            p.text(position, align, label, FontId::proportional(11.0), color);
        };
        text(
            Pos2::new(left, rect.top() + 9.0),
            Align2::LEFT_TOP,
            "ORIGINAL",
            INPUT,
        );
        text(
            Pos2::new(left + 78.0, rect.top() + 9.0),
            Align2::LEFT_TOP,
            "PROCESSED",
            OUTPUT,
        );
        text(
            Pos2::new(left, gain_top - 19.0),
            Align2::LEFT_TOP,
            "APPLIED ATTENUATION",
            REDUCTION,
        );
        for second in 0..=5 {
            let x = right - width * second as f32 / 5.12;
            p.line_segment(
                [
                    Pos2::new(x, rect.top() + 28.0),
                    Pos2::new(x, rect.bottom() - 24.0),
                ],
                Stroke::new(1.0, Color32::from_rgb(35, 42, 50)),
            );
            text(
                Pos2::new(x, rect.bottom() - 5.0),
                Align2::CENTER_BOTTOM,
                &format!("−{second}s"),
                MUTED,
            );
        }
        p.line_segment(
            [Pos2::new(left, center), Pos2::new(right, center)],
            Stroke::new(1.0, INPUT),
        );
        for (label, y) in [("0", gain_top), ("6 dB", gain_top + gain_height)] {
            p.line_segment(
                [Pos2::new(left, y), Pos2::new(right, y)],
                Stroke::new(1.0, Color32::from_rgb(50, 45, 64)),
            );
            text(Pos2::new(right, y), Align2::RIGHT_BOTTOM, label, REDUCTION);
        }
        let step = width / BINS as f32;
        let mut previous = None;
        for (i, bin) in self.bins.iter().enumerate() {
            let Some(bin) = bin else {
                previous = None;
                continue;
            };
            let x = left + i as f32 * step;
            for (visible, peak, color) in [
                (self.show_original, bin.input, INPUT),
                (self.show_processed, bin.output, OUTPUT),
            ] {
                if visible && peak > 0.0 {
                    let h = peak.min(1.0) * height;
                    p.line_segment(
                        [Pos2::new(x, center - h), Pos2::new(x, center + h)],
                        Stroke::new(step.max(1.0), color),
                    );
                }
            }
            let point = Pos2::new(x, gain_top + reduction(bin.gain) / 6.0 * gain_height);
            if let Some(old) = previous {
                p.line_segment([old, point], Stroke::new(1.5, REDUCTION));
            }
            previous = Some(point);
        }
        if self.bins.iter().all(Option::is_none) {
            text(
                Pos2::new((left + right) / 2.0, center - 12.0),
                Align2::CENTER_CENTER,
                "Play audio in your host to see the signal",
                MUTED,
            );
        }
        for (i, name, peak, color) in [
            (0, "IN", current.map_or(0.0, |b| b.input), INPUT),
            (1, "OUT", current.map_or(0.0, |b| b.output), OUTPUT),
        ] {
            let x = rect.right() - 65.0 + i as f32 * 34.0;
            text(
                Pos2::new(x + 7.0, rect.top() + 9.0),
                Align2::CENTER_TOP,
                name,
                MUTED,
            );
            let bar = Rect::from_min_max(
                Pos2::new(x, rect.top() + 31.0),
                Pos2::new(x + 14.0, rect.bottom() - 56.0),
            );
            p.rect_filled(bar, 2.0, Color32::from_rgb(41, 47, 55));
            let level = if fresh {
                ((db(peak) + 60.0) / 60.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            p.rect_filled(
                Rect::from_min_max(
                    Pos2::new(bar.left(), bar.bottom() - bar.height() * level),
                    bar.max,
                ),
                2.0,
                if peak > 1.0 && fresh {
                    Color32::from_rgb(249, 129, 113)
                } else {
                    color
                },
            );
            let label = if fresh && peak > 0.000001 {
                format!("{:.0}", db(peak))
            } else {
                "−∞".into()
            };
            text(
                Pos2::new(x + 7.0, rect.bottom() - 46.0),
                Align2::CENTER_TOP,
                &label,
                MUTED,
            );
        }
        text(
            Pos2::new(rect.right() - 41.0, rect.bottom() - 7.0),
            Align2::CENTER_BOTTOM,
            "dBFS",
            MUTED,
        );
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
        egui::Frame::NONE.inner_margin(egui::Margin::same(16)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 6.0);
            ui.horizontal(|ui| { ui.heading("CleanSlate Smart Gate");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{applied:.1} dB")).size(28.0).color(REDUCTION)); ui.label("Reduction"); }); });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Experimental speech gate").color(Color32::from_rgb(216, 181, 121)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(if !fresh { "No recent audio callback" } else if self.params.bypass.value() { "Bypassed" } else if !enabled { "Waiting · arm gate" } else if self.detection_state.load(Ordering::Acquire) == DetectionState::Waiting as u8 { "Waiting · original audio" } else if applied > 0.01 { "Attenuating" } else { "Passing original audio" }); }); });
            self.graph(ui, current, fresh);
            ui.horizontal(|ui| {
                let mut bypass = self.params.bypass.value(); let response = ui.checkbox(&mut bypass, "Bypass");
                if response.changed() && let Some(context) = &self.context { let setter = context.param_setter(); setter.begin_set_parameter(&self.params.bypass); setter.set_parameter(&self.params.bypass, bypass); setter.end_set_parameter(&self.params.bypass); }
                response.on_hover_text("Original audio. Arming does not turn bypass off.");
                let consent = self.consent.load(Ordering::Acquire); let armed = consent & CONSENT_MASK == Consent::Armed as u64;
                let response = ui.push_id(consent & !CONSENT_MASK, |ui| ui.add_enabled(consent & CONSENT_MASK != Consent::Unavailable as u64, egui::Button::new(if armed { "Disarm experimental gate" } else { "Arm experimental gate" }))).inner;
                if response.clicked() { let next = if armed { Consent::Ready } else { Consent::Armed }; let _ = self.consent.compare_exchange(consent, (consent & !CONSENT_MASK) | next as u64, Ordering::AcqRel, Ordering::Acquire); }
                ui.checkbox(&mut self.frozen, "Freeze").on_hover_text("Freeze history only. Processing and live meters continue.");
                ui.checkbox(&mut self.show_original, "Original"); ui.checkbox(&mut self.show_processed, "Processed"); });
            ui.horizontal(|ui| {
                let mut range = self.params.range.value(); let response = ui.add(egui::Slider::new(&mut range, 0.0..=6.0).text("Range").suffix(" dB"));
                if response.changed() && let Some(context) = &self.context { let setter = context.param_setter(); setter.begin_set_parameter(&self.params.range); setter.set_parameter(&self.params.range, range); setter.end_set_parameter(&self.params.range); }
                response.on_hover_text("Maximum attenuation. 0 dB preserves original audio; the experimental ceiling is 6 dB."); });
            let state = self.consent.load(Ordering::Acquire) & CONSENT_MASK;
            ui.label(if state == Consent::Unavailable as u64 { "Not prepared · original audio" }
                else if self.detection_state.load(Ordering::Acquire) == DetectionState::Failed as u8 { "Processing unavailable · original audio" }
                else if self.params.bypass.value() { "Bypassed · original audio" }
                else if state == Consent::Armed as u64 { if self.detection_state.load(Ordering::Acquire) == DetectionState::Current as u8 { "Last block · current detection data" } else { "Waiting for current detection data · original audio" } }
                else { "Not armed · original audio" });
            ui.label(RichText::new("Fixed: open 10% / close 1% · hold 300 ms · ramps 5 / 250 ms").small().color(MUTED));
            ui.label(RichText::new("Mono / stereo · 0 samples added latency · evidence expires at 100 ms").small().color(MUTED));
        });
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
}
