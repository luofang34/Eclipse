//! The page: the globe with its overlays, and the panel beside it.

use std::sync::Arc;

use eframe::egui::{self, Color32, Sense, Stroke};
use egui_wgpu::{WgpuConfiguration, WgpuSetupCreateNew};

use crate::map::{Gesture, MapView};

/// The page's GPU device, created without a display handle as the map renderer needs.
pub(crate) fn gpu_configuration() -> WgpuConfiguration {
    let setup = WgpuSetupCreateNew::without_display_handle();
    let descriptor = setup.device_descriptor.clone();
    let mut setup = setup;
    setup.device_descriptor = Arc::new(move |adapter| descriptor(adapter));
    WgpuConfiguration {
        wgpu_setup: setup.into(),
        ..Default::default()
    }
}

/// The App's state.
pub(crate) struct EclipseApp {
    host: egui_wgpu::RenderState,
    map: Option<MapView>,
}

impl EclipseApp {
    /// The App on the page's GPU.
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = cc
            .wgpu_render_state
            .clone()
            .ok_or("the page has no wgpu renderer")?;
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Ok(Self { host, map: None })
    }

    fn globe(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let ratio = f64::from(ui.ctx().pixels_per_point());
        let size = [
            (rect.width() * ui.ctx().pixels_per_point())
                .round()
                .max(1.0) as u32,
            (rect.height() * ui.ctx().pixels_per_point())
                .round()
                .max(1.0) as u32,
        ];
        if self.map.is_none() {
            match MapView::new(size, &self.host) {
                Ok(mut map) => {
                    map.resize(size, ratio);
                    if let Err(error) = map.look_at(40.0, -30.0, 0.6) {
                        tracing::error!(%error, "globe camera");
                    }
                    self.map = Some(map);
                }
                Err(error) => {
                    tracing::error!(%error, "globe unavailable");
                    ui.label(format!("The globe could not start: {error}"));
                    return;
                }
            }
        }
        let Some(map) = &mut self.map else { return };
        if map.size() != size {
            map.resize(size, ratio);
        }
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        // Gestures are in the camera's logical pixels, which egui points are.
        let to_pixels = |p: egui::Pos2| [f64::from(p.x - rect.min.x), f64::from(p.y - rect.min.y)];
        if response.dragged() {
            if let Some(now) = response.interact_pointer_pos() {
                let from = now - response.drag_delta();
                map.gesture(Gesture::Drag {
                    from: to_pixels(from),
                    to: to_pixels(now),
                });
            }
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                if let Some(at) = response.hover_pos() {
                    map.gesture(Gesture::Zoom {
                        at: to_pixels(at),
                        levels: f64::from(scroll) / 200.0,
                    });
                }
            }
        }
        match map.frame() {
            Ok(Some(texture)) => {
                ui.painter().image(
                    texture,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            Ok(None) => {}
            Err(error) => tracing::error!(%error, "globe frame"),
        }
        if let Some(projector) = map.projector(rect) {
            let painter = ui.painter_at(rect);
            for lat in [-89.0, -60.0, 0.0, 60.0, 85.0, 89.0] {
                let points: Vec<_> = (0..=360)
                    .filter_map(|lon| projector.project(lat, f64::from(lon) - 180.0))
                    .collect();
                for pair in points.windows(2) {
                    if pair[0].distance(pair[1]) < 40.0 {
                        painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, Color32::BLUE));
                    }
                }
            }
        }
        ui.ctx().request_repaint();
    }
}

impl eframe::App for EclipseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| self.globe(ui));
    }
}
