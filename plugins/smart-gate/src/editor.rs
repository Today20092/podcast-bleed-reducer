use crate::{CONSENT_MASK, Consent, GateParams};
use egui::{Color32, RichText};
use nice_plug::context::gui::GuiContext;
use nice_plug_egui::NiceEguiApp;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Panel {
    params: Arc<GateParams>,
    consent: Arc<AtomicU64>,
    context: Option<GuiContext>,
}

impl Panel {
    pub fn new(params: Arc<GateParams>, consent: Arc<AtomicU64>) -> Self {
        Self {
            params,
            consent,
            context: None,
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
    // Inherit the accepted CleanSlate plugin tokens, without depending on the desktop renderer.
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(32, 34, 38);
    visuals.override_text_color = Some(Color32::from_rgb(228, 231, 235));
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(52, 57, 64);
    visuals.selection.stroke = egui::Stroke::new(2.0, Color32::from_rgb(154, 197, 255));
    context.set_visuals(visuals);
}

impl Panel {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE.inner_margin(egui::Margin::same(24)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 16.0);
            ui.heading("CleanSlate Smart Gate");
            ui.label(RichText::new("Experimental speech gate").size(18.0).color(Color32::from_rgb(216, 181, 121)));
            ui.label("Arming allows up to 6 dB of attenuation when valid speech evidence is available. Bypass always preserves original audio.");
            ui.separator();
            let mut bypass = self.params.bypass.value();
            let response = ui.checkbox(&mut bypass, "Bypass");
            if response.changed() && let Some(context) = &self.context {
                    let setter = context.param_setter();
                    setter.begin_set_parameter(&self.params.bypass);
                    setter.set_parameter(&self.params.bypass, bypass);
                    setter.end_set_parameter(&self.params.bypass);
            }
            response.on_hover_text("Saved bypass is independent of temporary experimental arming.");
            let consent = self.consent.load(Ordering::Acquire);
            let state = consent & CONSENT_MASK;
            let armed = state == Consent::Armed as u64;
            let action = if armed { "Disarm experimental gate" } else { "Arm experimental gate" };
            let response = ui.push_id(consent & !CONSENT_MASK, |ui| {
                ui.add_enabled(state != Consent::Unavailable as u64, egui::Button::new(action))
            }).inner;
            if response.clicked() {
                let next = if armed { Consent::Ready } else { Consent::Armed };
                // The full token also rejects a lifecycle change during this frame.
                let _ = self.consent.compare_exchange(consent, (consent & !CONSENT_MASK) | next as u64, Ordering::AcqRel, Ordering::Acquire);
            }
            let state = self.consent.load(Ordering::Acquire) & CONSENT_MASK;
            ui.label(if state == Consent::Unavailable as u64 {
                "Not prepared · original audio"
            } else if bypass {
                "Bypassed · original audio"
            } else if state == Consent::Armed as u64 {
                "Armed · attenuation requires valid evidence"
            } else {
                "Not armed · original audio"
            });
            ui.label(RichText::new("Mono / stereo · 0 samples added latency").color(Color32::from_rgb(173, 181, 192)));
        });
    }
}
