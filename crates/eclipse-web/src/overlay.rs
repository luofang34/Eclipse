//! Drawing on the globe: curves, markers and shaded regions, projected through the camera
//! of the frame under them. Points behind the globe break a curve rather than join across.

use eclipse_core::geodesy::LatLon;
use eframe::egui::{Color32, Mesh, Painter, Pos2, Shape, Stroke, epaint::Vertex};

use crate::map::Projector;

/// Two consecutive points further apart than this on screen are not joined: the curve
/// passed behind the globe or off the view between them.
const MAX_SCREEN_STEP: f32 = 120.0;

/// How a curve is stroked.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Line {
    pub(crate) stroke: Stroke,
    pub(crate) dashed: bool,
}

/// Draws each polyline of `curves`.
pub(crate) fn curves(painter: &Painter, projector: &Projector, curves: &[Vec<LatLon>], line: Line) {
    for curve in curves {
        for run in runs(projector, curve) {
            if line.dashed {
                painter.extend(Shape::dashed_line(&run, line.stroke, 6.0, 5.0));
            } else {
                painter.add(Shape::line(run, line.stroke));
            }
        }
    }
}

/// The visible stretches of `curve` on screen.
fn runs(projector: &Projector, curve: &[LatLon]) -> Vec<Vec<Pos2>> {
    let mut runs: Vec<Vec<Pos2>> = vec![Vec::new()];
    for point in curve {
        let projected = projector.project(point.lat, point.lon);
        let Some(run) = runs.last_mut() else { continue };
        match projected {
            Some(p) if run.last().is_none_or(|q| q.distance(p) < MAX_SCREEN_STEP) => run.push(p),
            Some(p) => runs.push(vec![p]),
            None if !run.is_empty() => runs.push(Vec::new()),
            None => {}
        }
    }
    runs.retain(|run| run.len() > 1);
    runs
}

/// A filled region bounded by a closed curve, if it is wholly in view.
pub(crate) fn filled(painter: &Painter, projector: &Projector, outline: &[LatLon], fill: Color32) {
    let points: Option<Vec<Pos2>> = outline
        .iter()
        .map(|p| projector.project(p.lat, p.lon))
        .collect();
    if let Some(points) = points.filter(|points| points.len() > 2) {
        painter.add(Shape::convex_polygon(points, fill, Stroke::NONE));
    }
}

/// A star marker, as the Canon marks greatest eclipse.
pub(crate) fn asterisk(painter: &Painter, projector: &Projector, at: LatLon, color: Color32) {
    if let Some(c) = projector.project(at.lat, at.lon) {
        let stroke = Stroke::new(2.0, color);
        for i in 0..4 {
            let angle = f32::from(i as u8) * std::f32::consts::FRAC_PI_4;
            let d = eframe::egui::vec2(angle.cos(), angle.sin()) * 8.0;
            painter.line_segment([c - d, c + d], stroke);
        }
        painter.circle_stroke(c, 4.0, stroke);
    }
}

/// A cross marker, as the Canon marks the subsolar point.
pub(crate) fn cross(painter: &Painter, projector: &Projector, at: LatLon, color: Color32) {
    if let Some(c) = projector.project(at.lat, at.lon) {
        let stroke = Stroke::new(2.5, color);
        let d = 7.0;
        painter.line_segment(
            [c + eframe::egui::vec2(-d, -d), c + eframe::egui::vec2(d, d)],
            stroke,
        );
        painter.line_segment(
            [c + eframe::egui::vec2(-d, d), c + eframe::egui::vec2(d, -d)],
            stroke,
        );
    }
}

/// Shades the globe on a latitude and longitude grid `step` degrees apart, each vertex
/// coloured by `shade`; cells reaching behind the globe are left out.
pub(crate) fn shade_grid(
    painter: &Painter,
    projector: &Projector,
    step: f64,
    shade: impl Fn(LatLon) -> Color32,
) {
    let columns = (360.0 / step).round() as usize;
    let rows = (180.0 / step).round() as usize;
    let mut mesh = Mesh::default();
    let mut index: Vec<Option<u32>> = Vec::with_capacity((rows + 1) * (columns + 1));
    for row in 0..=rows {
        for column in 0..=columns {
            let at = LatLon {
                lat: -90.0 + row as f64 * step,
                lon: -180.0 + column as f64 * step,
            };
            index.push(projector.project(at.lat, at.lon).map(|pos| {
                mesh.vertices.push(Vertex {
                    pos,
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: shade(at),
                });
                (mesh.vertices.len() - 1) as u32
            }));
        }
    }
    let at = |row: usize, column: usize| index.get(row * (columns + 1) + column).copied().flatten();
    for row in 0..rows {
        for column in 0..columns {
            let corners = (
                at(row, column),
                at(row, column + 1),
                at(row + 1, column),
                at(row + 1, column + 1),
            );
            if let (Some(a), Some(b), Some(c), Some(d)) = corners {
                mesh.add_triangle(a, b, d);
                mesh.add_triangle(a, d, c);
            }
        }
    }
    add_mesh(painter, mesh);
}

/// Shades the half of the globe more than 90° from `centre`, as night is from the subsolar
/// point, with rings `step` degrees apart so the edge falls exactly on the great circle.
pub(crate) fn shade_far_side(
    painter: &Painter,
    projector: &Projector,
    centre: LatLon,
    fill: Color32,
) {
    const RINGS: usize = 30;
    const SEGMENTS: usize = 120;
    let anti = LatLon {
        lat: -centre.lat,
        lon: centre.lon + 180.0,
    };
    let mut mesh = Mesh::default();
    let mut index: Vec<Option<u32>> = Vec::new();
    for ring in 0..=RINGS {
        let radius = 90.0 * ring as f64 / RINGS as f64;
        for segment in 0..SEGMENTS {
            let bearing = 360.0 * segment as f64 / SEGMENTS as f64;
            let at = destination(anti, bearing, radius);
            index.push(projector.project(at.lat, at.lon).map(|pos| {
                mesh.vertices.push(Vertex {
                    pos,
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: fill,
                });
                (mesh.vertices.len() - 1) as u32
            }));
        }
    }
    let at = |ring: usize, segment: usize| {
        index
            .get(ring * SEGMENTS + segment % SEGMENTS)
            .copied()
            .flatten()
    };
    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let corners = (
                at(ring, segment),
                at(ring, segment + 1),
                at(ring + 1, segment),
                at(ring + 1, segment + 1),
            );
            if let (Some(a), Some(b), Some(c), Some(d)) = corners {
                mesh.add_triangle(a, b, d);
                mesh.add_triangle(a, d, c);
            }
        }
    }
    add_mesh(painter, mesh);
}

/// Adds `mesh` unless it has no triangle: close in, the grid's points can stand on screen
/// with no whole cell among them, and the painter rejects a mesh of vertices alone.
fn add_mesh(painter: &Painter, mesh: Mesh) {
    if !mesh.indices.is_empty() {
        painter.add(Shape::mesh(mesh));
    }
}

/// The great circle 90° from `centre`, where a body overhead at `centre` is on the horizon.
pub(crate) fn horizon_of(centre: LatLon) -> Vec<LatLon> {
    (0..=180)
        .map(|i| destination(centre, f64::from(i) * 2.0, 90.0))
        .collect()
}

/// The point `distance` degrees from `from` along the great circle leaving at `bearing`
/// degrees east of north, on the sphere.
pub(crate) fn destination(from: LatLon, bearing: f64, distance: f64) -> LatLon {
    let (lat, lon) = (from.lat.to_radians(), from.lon.to_radians());
    let (b, d) = (bearing.to_radians(), distance.to_radians());
    let lat2 = (lat.sin() * d.cos() + lat.cos() * d.sin() * b.cos())
        .clamp(-1.0, 1.0)
        .asin();
    let lon2 = lon + (b.sin() * d.sin() * lat.cos()).atan2(d.cos() - lat.sin() * lat2.sin());
    LatLon {
        lat: lat2.to_degrees(),
        lon: (lon2.to_degrees() + 540.0).rem_euclid(360.0) - 180.0,
    }
}
