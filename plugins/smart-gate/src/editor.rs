use crate::GateParams;
use egui::{Color32, RichText};
use nice_plug::context::gui::GuiContext;
use nice_plug_egui::NiceEguiApp;
use std::sync::Arc;

pub struct Panel {
    params: Arc<GateParams>,
    context: Option<GuiContext>,
}

impl Panel {
    pub fn new(params: Arc<GateParams>) -> Self {
        Self {
            params,
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
        // Inherit the accepted CleanSlate plugin tokens, without depending on the desktop renderer.
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(32, 34, 38);
        visuals.override_text_color = Some(Color32::from_rgb(228, 231, 235));
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(52, 57, 64);
        visuals.selection.stroke = egui::Stroke::new(2.0, Color32::from_rgb(154, 197, 255));
        context.set_visuals(visuals);
        self.context = Some(gui);
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.context = None;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut nice_plug_egui::Frame) {
        egui::Frame::NONE.inner_margin(egui::Margin::same(24)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 16.0);
            ui.heading("CleanSlate Smart Gate");
            ui.label(RichText::new("Pass-through prototype").size(18.0).color(Color32::from_rgb(216, 181, 121)));
            ui.label("Audio passes through unchanged. Speech detection and gating are not enabled in this build.");
            ui.separator();
            let mut bypass = self.params.bypass.value();
            let response = ui.checkbox(&mut bypass, "Bypass");
            if response.changed() && let Some(context) = &self.context {
                    let setter = context.param_setter();
                    setter.begin_set_parameter(&self.params.bypass);
                    setter.set_parameter(&self.params.bypass, bypass);
                    setter.end_set_parameter(&self.params.bypass);
            }
            response.on_hover_text("Both states preserve the input in this prototype. Bypass is saved with this plugin instance.");
            ui.label(if bypass { "Bypassed · original audio" } else { "Original audio · no gain reduction" });
            ui.label(RichText::new("Mono / stereo · 0 samples added latency").color(Color32::from_rgb(173, 181, 192)));
        });
    }
}
