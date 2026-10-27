//! What is drawn over the globe for the chosen eclipse at the time shown.

use eclipse_core::geodesy::{LatLon, overhead};
use eframe::egui::{Color32, Painter, Stroke};

use crate::{
    map::Projector,
    overlay::{self, Line},
    palette,
    selection::{Selected, VISIBILITY_STEP},
};

/// Draws the chosen eclipse's map at `now_tt`.
pub(crate) fn draw(painter: &Painter, projector: &Projector, selected: &Selected, now_tt: f64) {
    match selected {
        Selected::Solar { eclipse, map } => {
            if let Ok(e) = eclipse.elements(now_tt) {
                overlay::shade_far_side(painter, projector, overhead(e.sun), palette::NIGHT);
            }
            let solid = |color: Color32| Line {
                stroke: Stroke::new(2.0, color),
                dashed: false,
            };
            let dashed = |color: Color32| Line {
                stroke: Stroke::new(1.6, color),
                dashed: true,
            };
            overlay::curves(
                painter,
                projector,
                &map.penumbral_limits,
                solid(palette::PENUMBRA),
            );
            overlay::curves(painter, projector, &map.rise_set, solid(palette::PENUMBRA));
            overlay::curves(
                painter,
                projector,
                &map.max_rise_set,
                solid(palette::MAX_RISE_SET),
            );
            overlay::curves(
                painter,
                projector,
                &map.half_magnitude,
                dashed(palette::PENUMBRA),
            );
            overlay::curves(
                painter,
                projector,
                &map.umbral_limits,
                solid(palette::UMBRA),
            );
            overlay::curves(
                painter,
                projector,
                &map.central_line,
                dashed(palette::UMBRA),
            );
            if let Some(shadow) = selected.shadow_at(now_tt) {
                let edge = Line {
                    stroke: Stroke::new(1.2, palette::SHADOW_EDGE),
                    dashed: true,
                };
                overlay::curves(painter, projector, &shadow.penumbra, edge);
                for umbra in &shadow.umbra {
                    overlay::filled(painter, projector, umbra, palette::SHADOW);
                }
            }
            overlay::asterisk(painter, projector, eclipse.greatest, palette::PENUMBRA);
            overlay::cross(painter, projector, map.subsolar, palette::PENUMBRA);
        }
        Selected::Lunar {
            eclipse,
            visibility,
        } => {
            overlay::shade_grid(painter, projector, VISIBILITY_STEP, |at: LatLon| {
                let hidden = 1.0 - visibility.share(at);
                Color32::from_rgba_unmultiplied(0x30, 0x30, 0x38, (hidden * 150.0) as u8)
            });
            if let Ok(below) = eclipse.sublunar(now_tt) {
                let horizon = overlay::horizon_of(below);
                let line = Line {
                    stroke: Stroke::new(2.0, palette::MAX_RISE_SET),
                    dashed: false,
                };
                overlay::curves(painter, projector, &[horizon], line);
                overlay::cross(painter, projector, below, palette::MAX_RISE_SET);
            }
        }
    }
}
