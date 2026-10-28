//! What egui adds over the renderer's sky: the compass points on the horizon.

use eclipse_core::geodesy::LatLon;
use eframe::egui::{Align2, Color32, FontId, Painter, Pos2};
use maplibre::projection::globe::EARTH_RADIUS_METERS;

use super::SkyView;
use crate::{eye::EYE_HEIGHT_M, map::Projector};

/// How far along a direction the point projected for it lies, metres: any distance on the
/// ray from the eye projects alike.
const RAY_M: f64 = 50_000.0;

/// Draws over the sky seen from `view`.
pub(crate) fn draw(painter: &Painter, projector: &Projector, view: &SkyView) {
    compass(painter, projector, view.site);
}

/// Where the direction `d` from the eye at `site` shows.
fn screen(projector: &Projector, site: LatLon, d: [f64; 3]) -> Option<Pos2> {
    let (lat, lon) = (site.lat.to_radians(), site.lon.to_radians());
    let up = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()];
    let eye = up.map(|u| u * (EARTH_RADIUS_METERS + EYE_HEIGHT_M));
    let p = [0, 1, 2].map(|i| eye[i] + d[i] * RAY_M);
    let distance = p.iter().map(|c| c * c).sum::<f64>().sqrt();
    projector.project_at(
        (p[2] / distance).asin().to_degrees(),
        p[1].atan2(p[0]).to_degrees(),
        distance - EARTH_RADIUS_METERS,
    )
}

/// N, E, S and W on the horizon.
fn compass(painter: &Painter, projector: &Projector, site: LatLon) {
    let lon = site.lon.to_radians();
    let up = eclipse_core::geodesy::up(site);
    let east = [-lon.sin(), lon.cos(), 0.0];
    let north = [
        up[1] * east[2] - up[2] * east[1],
        up[2] * east[0] - up[0] * east[2],
        up[0] * east[1] - up[1] * east[0],
    ];
    for (name, d) in [
        ("N", north),
        ("E", east),
        ("S", north.map(|c| -c)),
        ("W", east.map(|c| -c)),
    ] {
        // Lifted a little, so the letter stands on the horizon rather than in it.
        let lifted = [0, 1, 2].map(|i| d[i] + up[i] * 0.02);
        if let Some(p) = screen(projector, site, lifted) {
            painter.text(
                p,
                Align2::CENTER_BOTTOM,
                name,
                FontId::proportional(18.0),
                Color32::from_rgb(0xf0, 0xe0, 0x90),
            );
        }
    }
}
