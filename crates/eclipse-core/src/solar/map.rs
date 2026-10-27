//! The curves of a global eclipse map, as the NASA Five Millennium Canon draws them.
//!
//! Every curve is traced through time. A limit of a shadow cone is where its edge grazes
//! the Earth: the points on the edge that, held fixed, neither enter nor leave the cone at
//! that instant. Where the Sun rises or sets the points lie on the Earth's outline seen
//! from the Sun, where the penumbra's edge passes (the eclipse begins or ends with the Sun
//! on the horizon) or where the magnitude stops growing (maximum eclipse on the horizon).

use super::{SolarEclipse, trace};
use crate::{
    EclipseError,
    besselian::{Cone, Elements},
    geodesy::{LatLon, cross, dot, lat_lon, sub},
};

/// The half-interval of the time differences that find where points stop entering, days.
const H: f64 = 30.0 / 86_400.0;
/// The time step curves are traced at, days.
const STEP: f64 = 60.0 / 86_400.0;
/// Angles sampled around a cone's edge.
const EDGE_SAMPLES: usize = 180;
/// Angles sampled around the Earth's outline.
const OUTLINE_SAMPLES: usize = 360;

/// The curves of a solar eclipse map, each a list of polylines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SolarMap {
    /// The northern and southern limits of the penumbra: magnitude zero at maximum.
    pub penumbral_limits: Vec<Vec<LatLon>>,
    /// Where the eclipse begins or ends with the Sun on the horizon.
    pub rise_set: Vec<Vec<LatLon>>,
    /// Where maximum eclipse comes with the Sun on the horizon.
    pub max_rise_set: Vec<Vec<LatLon>>,
    /// Where the magnitude at maximum is one half.
    pub half_magnitude: Vec<Vec<LatLon>>,
    /// The limits of the path of totality or annularity.
    pub umbral_limits: Vec<Vec<LatLon>>,
    /// The central line, the shadow axis on the ground.
    pub central_line: Vec<Vec<LatLon>>,
    /// Where the Sun stands overhead at greatest eclipse.
    pub subsolar: LatLon,
}

/// The shadow on the ground at one instant.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShadowOutline {
    /// The penumbra's edge on the day side.
    pub penumbra: Vec<Vec<LatLon>>,
    /// The umbra's or antumbra's edge.
    pub umbra: Vec<Vec<LatLon>>,
}

impl SolarEclipse {
    /// Traces the map's curves.
    pub fn map(&self) -> Result<SolarMap, EclipseError> {
        let (start, end) = self.span();
        let (start, end) = (start + H, end - H);
        let traced = |at: &dyn Fn(f64) -> Result<Vec<[f64; 3]>, EclipseError>| {
            trace::sweep(start, end, STEP, at).map(|instants| to_lat_lon(trace::link(instants)))
        };
        let e = self.elements(self.greatest_tt)?;
        Ok(SolarMap {
            penumbral_limits: traced(&|t| flanks(self, t, Elements::penumbra))?,
            rise_set: traced(&|t| Ok(outline_contacts(&self.elements(t)?)))?,
            max_rise_set: traced(&|t| outline_maxima(self, t))?,
            half_magnitude: traced(&|t| flanks(self, t, |e| e.magnitude_cone(0.5)))?,
            umbral_limits: if self.kind == super::SolarKind::Partial {
                Vec::new()
            } else {
                traced(&|t| flanks(self, t, Elements::umbra))?
            },
            central_line: traced(&|t| {
                let e = self.elements(t)?;
                Ok(e.surface(e.x, e.y).into_iter().collect())
            })?,
            subsolar: subsolar(e.sun),
        })
    }

    /// The edges of the penumbra and umbra on the ground at `jd_tt`.
    pub fn shadow_outline(&self, jd_tt: f64) -> Result<ShadowOutline, EclipseError> {
        let e = self.elements(jd_tt)?;
        let ring = |cone: Cone| {
            let mut runs: Vec<Vec<LatLon>> = vec![Vec::new()];
            for i in 0..=EDGE_SAMPLES {
                let phi = i as f64 * std::f64::consts::TAU / EDGE_SAMPLES as f64;
                match (edge_point(&e, cone, phi), runs.last_mut()) {
                    (Some(p), Some(run)) => run.push(lat_lon(p)),
                    (None, Some(run)) if !run.is_empty() => runs.push(Vec::new()),
                    _ => {}
                }
            }
            runs.retain(|run| run.len() > 1);
            runs
        };
        Ok(ShadowOutline {
            penumbra: ring(e.penumbra()),
            umbra: if self.kind == super::SolarKind::Partial {
                Vec::new()
            } else {
                ring(e.umbra())
            },
        })
    }
}

/// The two limits of `cone` either side of the central line, over five minutes either side
/// of `jd_tt`, each as a polyline; empty where the axis misses the Earth.
pub(super) fn limits_near(
    eclipse: &SolarEclipse,
    jd_tt: f64,
    cone_of: fn(&Elements) -> Cone,
) -> Result<[Vec<LatLon>; 2], EclipseError> {
    let axis_point = |t: f64| -> Result<Option<[f64; 3]>, EclipseError> {
        let e = eclipse.elements(t)?;
        Ok(e.surface(e.x, e.y))
    };
    let step = 10.0 / 86_400.0;
    let (Some(centre), Some(before), Some(after)) = (
        axis_point(jd_tt)?,
        axis_point(jd_tt - step)?,
        axis_point(jd_tt + step)?,
    ) else {
        return Ok([Vec::new(), Vec::new()]);
    };
    // Which side of the path a point lies on: across the path's direction at the centre.
    let across = cross(sub(after, before), centre);
    let mut sides: [Vec<LatLon>; 2] = [Vec::new(), Vec::new()];
    let (start, end) = eclipse.span();
    for i in -30..=30 {
        let t = jd_tt + f64::from(i) * step;
        if t - H < start || t + H > end {
            continue;
        }
        for p in flanks(eclipse, t, cone_of)? {
            let side = usize::from(dot(sub(p, centre), across) < 0.0);
            if let Some(list) = sides.get_mut(side) {
                list.push(lat_lon(p));
            }
        }
    }
    Ok(sides)
}

/// The points where the edge of `cone` grazes the Earth at `jd_tt`.
pub(super) fn flanks(
    eclipse: &SolarEclipse,
    jd_tt: f64,
    cone_of: fn(&Elements) -> Cone,
) -> Result<Vec<[f64; 3]>, EclipseError> {
    let (before, now, after) = (
        eclipse.elements(jd_tt - H)?,
        eclipse.elements(jd_tt)?,
        eclipse.elements(jd_tt + H)?,
    );
    let cone = cone_of(&now);
    // How far inside the cone a fixed point is, as a difference of squares.
    let inside = |e: &Elements, p: [f64; 3]| {
        let shadow = e.shadow(p);
        cone_of(e).radius(shadow.zeta).powi(2) - shadow.delta.powi(2)
    };
    let roots = trace::circular_roots(EDGE_SAMPLES, |phi| {
        let p = edge_point(&now, cone, phi)?;
        Some(inside(&after, p) - inside(&before, p))
    });
    Ok(roots
        .into_iter()
        .filter_map(|phi| edge_point(&now, cone, phi))
        .collect())
}

/// The point of the day side on the edge of `cone` at angle `phi` around the axis.
fn edge_point(e: &Elements, cone: Cone, phi: f64) -> Option<[f64; 3]> {
    let mut zeta = 0.0;
    let mut point = None;
    // The radius depends on the height it is taken at; a few rounds settle it.
    for _ in 0..5 {
        let r = cone.radius(zeta).abs();
        let p = e.surface(e.x + r * phi.cos(), e.y + r * phi.sin())?;
        zeta = e.project(p)[2];
        point = Some(p);
    }
    point
}

/// Where the penumbra's edge crosses the Earth's outline at the instant of `e`.
fn outline_contacts(e: &Elements) -> Vec<[f64; 3]> {
    trace::circular_roots(OUTLINE_SAMPLES, |theta| {
        let shadow = e.shadow(e.outline(theta));
        Some(shadow.delta - shadow.l1)
    })
    .into_iter()
    .map(|theta| e.outline(theta))
    .collect()
}

/// Where a point of the outline inside the penumbra has its maximum at `jd_tt`.
fn outline_maxima(eclipse: &SolarEclipse, jd_tt: f64) -> Result<Vec<[f64; 3]>, EclipseError> {
    let (before, now, after) = (
        eclipse.elements(jd_tt - H)?,
        eclipse.elements(jd_tt)?,
        eclipse.elements(jd_tt + H)?,
    );
    let roots = trace::circular_roots(OUTLINE_SAMPLES, |theta| {
        let p = now.outline(theta);
        (now.shadow(p).magnitude() > 0.0)
            .then(|| after.shadow(p).magnitude() - before.shadow(p).magnitude())
    });
    Ok(roots.into_iter().map(|theta| now.outline(theta)).collect())
}

/// The subsolar point: where the ellipsoid's normal points at the Sun.
fn subsolar(sun: [f64; 3]) -> LatLon {
    let distance = crate::geodesy::norm(sun);
    LatLon {
        lat: (sun[2] / distance).asin().to_degrees(),
        lon: sun[1].atan2(sun[0]).to_degrees(),
    }
}

fn to_lat_lon(curves: Vec<Vec<[f64; 3]>>) -> Vec<Vec<LatLon>> {
    curves
        .into_iter()
        .map(|curve| curve.into_iter().map(lat_lon).collect())
        .collect()
}
