//! What egui adds over the renderer's sky. The renderer lights the sky by the Sun's
//! elevation alone and the Moon by the Sun's direction alone, so the darkening as the Sun
//! is covered, the new Moon's silhouette on the Sun, the corona in totality and the Earth's
//! shadow on an eclipsed Moon are drawn here, with the compass points on the horizon.

use eclipse_core::{
    LunarEclipse, SolarEclipse,
    geodesy::{LatLon, surface},
};
use eframe::egui::{
    Align2, Color32, FontId, Mesh, Painter, Pos2, Rect, Shape, Stroke, epaint::Vertex,
};
use maplibre::projection::globe::EARTH_RADIUS_METERS;

use super::{EYE_HEIGHT_M, SkyView};
use crate::{
    map::{Projector, SkyFrame},
    selection::Selected,
};

/// How far along a direction the point projected for it lies, metres: any distance on the
/// ray from the eye projects alike.
const RAY_M: f64 = 50_000.0;
/// Steps around a circle drawn on the sky.
const CIRCLE_STEPS: usize = 96;
/// The darkest the sky gets in totality, as a share of full black.
const TOTALITY_DARKNESS: f32 = 0.88;
const ARCMINUTE: f64 = std::f64::consts::PI / 10_800.0;
/// The step on the sky that finds which way east and north lie on the screen, arcminutes.
const STEP_ARCMIN: f64 = 2.0;

/// Draws over the sky seen from `view` at `now_tt`.
pub(crate) fn draw(
    painter: &Painter,
    projector: &Projector,
    rect: Rect,
    view: &SkyView,
    sky: &SkyFrame,
    selected: &Selected,
    now_tt: f64,
) {
    let fov = view.field_of_view.to_radians();
    let per_radian = f64::from(rect.height()) / 2.0 / (fov / 2.0).tan();
    let place = |d: [f64; 3]| screen(projector, view.site, d);
    let sun_elevation = sky
        .sun
        .map_or(-90.0, |(d, _)| super::horizontal(view.site, d)[1]);
    ground(painter, rect, view, sun_elevation, &place);
    match selected {
        Selected::Solar { eclipse, .. } => {
            if let Some((sun, radius)) = sky.sun {
                let disc = |d: [f64; 3], radius: f64| {
                    place(d).map(|c| (c, (radius.to_radians() * per_radian) as f32))
                };
                let moon = sky.moon.and_then(|(d, radius)| disc(d, radius));
                solar(
                    painter,
                    rect,
                    eclipse,
                    view.site,
                    disc(sun, radius),
                    moon,
                    now_tt,
                );
            }
        }
        Selected::Lunar { eclipse, .. } => {
            if let Some((moon, radius)) = sky.moon {
                let r = radius.to_radians() * per_radian;
                lunar(painter, eclipse, moon, r, per_radian, &place, now_tt);
            }
        }
    }
    compass(painter, projector, view.site);
}

/// The ground below the horizon, which the renderer leaves open to the sky when no terrain
/// is drawn. The view never rolls, so the horizon is a level line across the screen.
fn ground(
    painter: &Painter,
    rect: Rect,
    view: &SkyView,
    sun_elevation: f64,
    place: &dyn Fn([f64; 3]) -> Option<Pos2>,
) {
    let up = eclipse_core::geodesy::up(view.site);
    let lon = view.site.lon.to_radians();
    let east = [-lon.sin(), lon.cos(), 0.0];
    let north = cross(up, east);
    let level = |azimuth: f64| {
        let a = azimuth.to_radians();
        place([0, 1, 2].map(|i| east[i] * a.sin() + north[i] * a.cos()))
    };
    let (Some(a), Some(b)) = (level(view.azimuth - 20.0), level(view.azimuth + 20.0)) else {
        return;
    };
    if (b.x - a.x).abs() < 1e-3 {
        return;
    }
    let at = |x: f32| a.y + (b.y - a.y) * (x - a.x) / (b.x - a.x);
    let (left, right) = (at(rect.left()), at(rect.right()));
    if left.min(right) >= rect.bottom() {
        return;
    }
    // Daylit ground, dimming through twilight to night.
    let light = ((sun_elevation + 6.0) / 12.0).clamp(0.0, 1.0) as f32;
    let shade =
        |day: u8, night: u8| (f32::from(night) + (f32::from(day) - f32::from(night)) * light) as u8;
    let color = Color32::from_rgb(shade(0x4a, 0x08), shade(0x50, 0x09), shade(0x40, 0x0c));
    let below = vec![
        Pos2::new(rect.left(), left.max(rect.top())),
        Pos2::new(rect.right(), right.max(rect.top())),
        rect.right_bottom(),
        rect.left_bottom(),
    ];
    painter.add(Shape::convex_polygon(below, color, Stroke::NONE));
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

/// The daylight dimming as the Moon covers the Sun, and the corona in totality.
fn solar(
    painter: &Painter,
    rect: Rect,
    eclipse: &SolarEclipse,
    site: LatLon,
    sun: Option<(Pos2, f32)>,
    moon: Option<(Pos2, f32)>,
    now_tt: f64,
) {
    let Ok(elements) = eclipse.elements(now_tt) else {
        return;
    };
    let shadow = elements.shadow(surface(site));
    if shadow.zeta <= 0.0 {
        return;
    }
    let covered = shadow.obscuration() as f32;
    let total = shadow.central() && shadow.l2 < 0.0;
    // The sky stays bright until most of the Sun is covered, then falls fast.
    let darkness = if total {
        TOTALITY_DARKNESS
    } else {
        TOTALITY_DARKNESS * covered.powi(6)
    };
    let shade = Color32::from_black_alpha((darkness * 255.0) as u8);
    let reach = rect.width().hypot(rect.height());
    match sun {
        // The uncovered Sun keeps its glare: the dimming stops at its limb.
        Some((c, r)) if !total => painter.add(ring(c, r * 1.02, reach, shade, shade)),
        _ => painter.rect_filled(rect, 0.0, shade),
    };
    // The new Moon shows only against the Sun, as a silhouette.
    if let (Some(sun), Some(moon)) = (sun, moon) {
        let lens = overlap(sun, moon);
        if lens.len() > 2 {
            let night = Color32::from_rgb(0x06, 0x06, 0x0a);
            painter.add(Shape::convex_polygon(lens, night, Stroke::NONE));
        }
    }
    if let (true, Some((c, r))) = (total, sun) {
        let pearl = Color32::from_rgba_unmultiplied(0xf4, 0xf0, 0xe6, 0xd0);
        painter.add(ring(c, r, r * 2.6, pearl, Color32::TRANSPARENT));
    }
}

/// The Earth's penumbra and umbra on the Moon's disc of `radius` pixels.
fn lunar(
    painter: &Painter,
    eclipse: &LunarEclipse,
    moon: [f64; 3],
    radius: f64,
    per_radian: f64,
    place: &dyn Fn([f64; 3]) -> Option<Pos2>,
    now_tt: f64,
) {
    let (Ok(at), Ok([penumbra, umbra, _])) =
        (eclipse.moon_in_shadow(now_tt), eclipse.radii(now_tt))
    else {
        return;
    };
    let Some(centre) = place(moon) else { return };
    // East and north on the sky at the Moon, as screen steps per arcminute. The shadow's
    // centre, often off the screen, lies opposite the Moon's offset from it.
    let east = normalize(cross([0.0, 0.0, 1.0], moon));
    let north = cross(moon, east);
    let step = |axis: [f64; 3]| {
        let d = normalize([0, 1, 2].map(|i| moon[i] + axis[i] * STEP_ARCMIN * ARCMINUTE));
        place(d).map(|p| (p - centre) / STEP_ARCMIN as f32)
    };
    let (Some(e), Some(n)) = (step(east), step(north)) else {
        return;
    };
    let shadow = centre - e * at.east as f32 - n * at.north as f32;
    let moon_disc = (centre, radius as f32);
    let px = |arcmin: f64| (arcmin * ARCMINUTE * per_radian) as f32;
    let dim = Color32::from_black_alpha(70);
    let copper = Color32::from_rgba_unmultiplied(0x6a, 0x12, 0x08, 0xe8);
    for (disc, fill) in [((shadow, px(penumbra)), dim), ((shadow, px(umbra)), copper)] {
        let lens = overlap(moon_disc, disc);
        if lens.len() > 2 {
            painter.add(Shape::convex_polygon(lens, fill, Stroke::NONE));
        }
    }
}

/// The convex region where two discs overlap, as a polygon.
fn overlap(a: (Pos2, f32), b: (Pos2, f32)) -> Vec<Pos2> {
    let inside = |p: Pos2, (c, r): (Pos2, f32)| p.distance(c) <= r + 0.01;
    let mut points: Vec<Pos2> = circle(a)
        .filter(|p| inside(*p, b))
        .chain(circle(b).filter(|p| inside(*p, a)))
        .collect();
    if points.is_empty() {
        return points;
    }
    let n = points.len() as f32;
    let middle = points.iter().fold(Pos2::ZERO, |m, p| m + p.to_vec2() / n);
    points.sort_by(|p, q| {
        let angle = |v: Pos2| (v.y - middle.y).atan2(v.x - middle.x);
        angle(*p).total_cmp(&angle(*q))
    });
    points
}

/// Points around a circle, about two points apart, so a large circle's arc across a small
/// one stays round.
fn circle((c, r): (Pos2, f32)) -> impl Iterator<Item = Pos2> {
    let steps = ((std::f32::consts::PI * r) as usize).clamp(CIRCLE_STEPS, 8192);
    (0..steps).map(move |i| {
        let angle = std::f32::consts::TAU * i as f32 / steps as f32;
        c + eframe::egui::vec2(angle.cos(), angle.sin()) * r
    })
}

/// A ring around `c` from radius `inner` to `outer`, coloured from `from` to `to`.
fn ring(c: Pos2, inner: f32, outer: f32, from: Color32, to: Color32) -> Shape {
    let mut mesh = Mesh::default();
    for i in 0..CIRCLE_STEPS {
        let angle = std::f32::consts::TAU * i as f32 / CIRCLE_STEPS as f32;
        let p = c + eframe::egui::vec2(angle.cos(), angle.sin());
        let d = p - c;
        for (radius, color) in [(inner, from), (outer, to)] {
            mesh.vertices.push(Vertex {
                pos: c + d * radius,
                uv: eframe::egui::epaint::WHITE_UV,
                color,
            });
        }
        let (a, b) = (2 * i as u32, (2 * ((i + 1) % CIRCLE_STEPS)) as u32);
        mesh.add_triangle(a, a + 1, b + 1);
        mesh.add_triangle(a, b + 1, b);
    }
    Shape::mesh(mesh)
}

/// N, E, S and W on the horizon.
fn compass(painter: &Painter, projector: &Projector, site: LatLon) {
    let lon = site.lon.to_radians();
    let up = eclipse_core::geodesy::up(site);
    let east = [-lon.sin(), lon.cos(), 0.0];
    let north = cross(up, east);
    for (name, d) in [
        ("N", north),
        ("E", east),
        ("S", north.map(|c| -c)),
        ("W", east.map(|c| -c)),
    ] {
        // Lifted a little, so the letter stands on the horizon rather than in it.
        let lifted = normalize([0, 1, 2].map(|i| d[i] + up[i] * 0.02));
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

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let length = a.iter().map(|c| c * c).sum::<f64>().sqrt().max(1e-12);
    a.map(|c| c / length)
}
