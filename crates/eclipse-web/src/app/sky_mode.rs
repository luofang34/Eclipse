//! The globe's area while the eye stands on the ground: the flight down, looking around and
//! zooming the field of view, and the sky drawn for the time shown.

use eframe::egui::{self, Color32, RichText};

use super::EclipseApp;
use crate::{map::GLOBE_FIELD_OF_VIEW, panel, sky, sky::Body};

/// The narrowest and widest field of view the viewer can zoom to, degrees.
const FIELD_OF_VIEW: std::ops::RangeInclusive<f64> = 0.5..=100.0;

impl EclipseApp {
    /// Draws the view from the ground into `rect`; true while it needs more frames.
    pub(super) fn sky_globe(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
    ) -> bool {
        let (Some(view), Some(map), Some(selected)) =
            (&mut self.sky, &mut self.map, &self.selected)
        else {
            return false;
        };
        let now_s = ui.input(|input| input.time);
        let mut moved = false;
        if response.dragged() {
            let per_point = view.field_of_view / f64::from(rect.height());
            let d = response.drag_delta();
            view.look([-f64::from(d.x) * per_point, f64::from(d.y) * per_point]);
            moved = true;
        }
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if response.hovered() && scroll != 0.0 {
            let zoomed = view.field_of_view * (-f64::from(scroll) / 300.0).exp();
            view.field_of_view = zoomed.clamp(*FIELD_OF_VIEW.start(), *FIELD_OF_VIEW.end());
            moved = true;
        }
        let frame = view.sky(self.now_tt);
        view.track(&frame);
        let pose = view.pose(now_s);
        map.set_field_of_view(view.frame_field_of_view(now_s));
        if let Err(error) = map.set_pose(pose) {
            tracing::error!(%error, "standing pose");
        }
        // A new Moon lit only by earthshine would show against the daylit sky; its
        // silhouette on the Sun is drawn over the frame instead.
        let solar = matches!(selected, crate::selection::Selected::Solar { .. });
        map.set_sky(Some(crate::map::SkyFrame {
            moon: frame.moon.filter(|_| !solar),
            ..frame.clone()
        }));
        let mut waiting = false;
        match map.frame() {
            Ok(Some(texture)) => {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture, rect, uv, Color32::WHITE);
            }
            Ok(None) => waiting = true,
            Err(error) => tracing::error!(%error, "sky frame"),
        }
        if !view.flying()
            && let Some(projector) = map.projector(rect)
        {
            let painter = ui.painter_at(rect);
            sky::draw(
                &painter,
                &projector,
                rect,
                view,
                &frame,
                selected,
                self.now_tt,
            );
        }
        moved || waiting || view.flying() || map.needs_redraw()
    }

    /// The panel's account of the view from the ground, and its controls.
    pub(super) fn sky_panel(&mut self, ui: &mut egui::Ui) {
        let Some(view) = &mut self.sky else { return };
        ui.label(RichText::new(format!("Standing at {}", panel::position(view.site))).strong());
        let frame = view.sky(self.now_tt);
        for (name, body) in [("Sun", frame.sun), ("Moon", frame.moon)] {
            if let Some((direction, _)) = body {
                let [azimuth, elevation] = sky::horizontal(view.site, direction);
                ui.label(format!("{name}: {elevation:.1}° up, azimuth {azimuth:.0}°"));
            }
        }
        let body = match self.selected {
            Some(crate::selection::Selected::Lunar { .. }) => Body::Moon,
            _ => Body::Sun,
        };
        let mut follow = view.follow.is_some();
        if ui
            .checkbox(&mut follow, format!("Keep the {} in view", body_name(body)))
            .changed()
        {
            view.follow = follow.then_some(body);
        }
        ui.add(
            egui::Slider::new(&mut view.field_of_view, FIELD_OF_VIEW)
                .logarithmic(true)
                .suffix("°")
                .text("field of view"),
        );
        ui.label("Drag to look around, scroll to zoom.");
        if ui.button("Back to the globe").clicked() {
            self.leave_sky();
        }
    }

    /// Returns from the ground to the globe over the eclipse.
    pub(super) fn leave_sky(&mut self) {
        self.sky = None;
        if let Some(map) = &mut self.map {
            map.set_sky(None);
            map.set_field_of_view(GLOBE_FIELD_OF_VIEW);
        }
        self.look = self.selected.as_ref().map(|s| s.focus());
    }
}

fn body_name(body: Body) -> &'static str {
    match body {
        Body::Sun => "Sun",
        Body::Moon => "Moon",
    }
}
