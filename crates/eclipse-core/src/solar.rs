//! A solar eclipse: when and where it is greatest, its type, and the catalogue quantities
//! the NASA Five Millennium Canon lists for it.

use navigate_celestial::DeltaT;

use crate::{
    EclipseError, Track,
    besselian::{Elements, Shadow},
    bodies,
    geodesy::{self, LatLon, altitude, lat_lon},
    saros, time,
};

mod local;
mod map;
mod trace;

pub use local::{Contact, LocalSolar};
pub use map::{ShadowOutline, SolarMap};

/// Half the span sampled around greatest eclipse, days; the penumbra crosses the Earth in
/// at most about six and a half hours.
const HALF_SPAN: f64 = 4.5 / 24.0;

/// What kind of solar eclipse it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolarKind {
    /// The umbra misses the Earth.
    Partial,
    /// The antumbra reaches the Earth: a ring of Sun stays around the Moon.
    Annular,
    /// The umbra reaches the Earth.
    Total,
    /// Total along part of the central line and annular along the rest.
    Hybrid,
}

impl SolarKind {
    /// The Canon's one-letter code.
    pub fn code(self) -> char {
        match self {
            Self::Partial => 'P',
            Self::Annular => 'A',
            Self::Total => 'T',
            Self::Hybrid => 'H',
        }
    }

    /// The name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Partial => "Partial",
            Self::Annular => "Annular",
            Self::Total => "Total",
            Self::Hybrid => "Hybrid",
        }
    }
}

/// A solar eclipse and the circumstances of its greatest eclipse.
#[derive(Clone, Debug)]
pub struct SolarEclipse {
    track: Track,
    /// Greatest eclipse, when the shadow axis passes closest to the Earth's centre, TT
    /// Julian date.
    pub greatest_tt: f64,
    /// Delta T at greatest eclipse.
    pub delta_t: DeltaT,
    /// Its type.
    pub kind: SolarKind,
    /// Whether the shadow axis meets the Earth.
    pub central: bool,
    /// The axis's least distance from the Earth's centre in equatorial radii, positive
    /// when it passes north of it.
    pub gamma: f64,
    /// For a central eclipse the Moon's apparent diameter over the Sun's; otherwise the
    /// fraction of the Sun's diameter covered at the point of greatest eclipse.
    pub magnitude: f64,
    /// Where the eclipse is greatest: on the axis, or for a non-central eclipse at the
    /// edge of the day side nearest the axis.
    pub greatest: LatLon,
    /// The Sun's altitude there, degrees.
    pub sun_altitude: f64,
    /// The width of the path of totality or annularity there, km.
    pub path_width_km: Option<f64>,
    /// The duration of totality or annularity there, seconds.
    pub central_duration_s: Option<f64>,
    /// The Canon's lunation number.
    pub lunation: i64,
    /// The Saros series.
    pub saros: i64,
}

impl SolarEclipse {
    /// The solar eclipse at the new moon nearest `jd_tt`, or `None` if there is none.
    pub fn near(jd_tt: f64) -> Result<Option<Self>, EclipseError> {
        let coarse = closest_approach(jd_tt)?;
        // The penumbra misses the Earth by a margin the refinement cannot close.
        let rough = Elements::new(&bodies(coarse)?);
        if rough.x.hypot(rough.y) > 1.0 + rough.l1 + 0.02 {
            return Ok(None);
        }
        let track = Track::sample(coarse - HALF_SPAN, coarse + HALF_SPAN)?;
        let gamma_at = |t: f64| -> Result<f64, EclipseError> {
            let e = Elements::new(&track.at(t)?);
            Ok(e.x.hypot(e.y))
        };
        let greatest_tt = golden_min(coarse - 0.08, coarse + 0.08, 1e-7, gamma_at)?;
        let e = Elements::new(&track.at(greatest_tt)?);
        let edge = nearest_day_side(&e);
        let shadow = e.shadow(edge.point);
        if edge.distance >= e.l1 {
            return Ok(None);
        }
        let central = edge.distance == 0.0;
        let kind = if central {
            central_kind(&track)?
        } else if edge.distance < e.l2.abs() {
            if shadow.l2 < 0.0 {
                SolarKind::Total
            } else {
                SolarKind::Annular
            }
        } else {
            SolarKind::Partial
        };
        let greatest = lat_lon(edge.point);
        let magnitude = if central {
            shadow.diameter_ratio()
        } else {
            shadow.magnitude()
        };
        let mut eclipse = Self {
            track,
            greatest_tt,
            delta_t: time::delta_t_at(greatest_tt),
            kind,
            central,
            gamma: e.x.hypot(e.y) * e.y.signum(),
            magnitude,
            greatest,
            sun_altitude: altitude(greatest, e.sun),
            path_width_km: None,
            central_duration_s: None,
            lunation: saros::solar_lunation(greatest_tt),
            saros: 0,
        };
        eclipse.saros = saros::solar_saros(eclipse.lunation);
        if kind != SolarKind::Partial {
            eclipse.central_duration_s = eclipse
                .local(greatest)?
                .and_then(|local| local.central_duration_s);
            eclipse.path_width_km = eclipse.path_width(greatest_tt)?;
        }
        Ok(Some(eclipse))
    }

    /// The Besselian elements at `jd_tt`, within the sampled span.
    pub fn elements(&self, jd_tt: f64) -> Result<Elements, EclipseError> {
        Ok(Elements::new(&self.track.at(jd_tt)?))
    }

    /// The sampled span, TT Julian dates.
    pub fn span(&self) -> (f64, f64) {
        self.track.window()
    }

    /// The width of the umbral path through the greatest eclipse, at right angles to the
    /// path: the sum of the distances from that point to the two limits.
    fn path_width(&self, jd_tt: f64) -> Result<Option<f64>, EclipseError> {
        let limits = map::limits_near(self, jd_tt, Elements::umbra)?;
        let [a, b] = limits;
        Ok((a.len() > 1 && b.len() > 1).then(|| {
            geodesy::distance_to_polyline(self.greatest, &a)
                + geodesy::distance_to_polyline(self.greatest, &b)
        }))
    }
}

/// The instant near `jd_tt` when the shadow axis passes closest to the Earth's centre,
/// to about a minute, from Navigate directly.
fn closest_approach(jd_tt: f64) -> Result<f64, EclipseError> {
    let gamma = |t: f64| -> Result<f64, EclipseError> {
        let e = Elements::new(&bodies(t)?);
        Ok(e.x.hypot(e.y))
    };
    // The mean new moon lies within about 14 hours of the true one.
    let mut best = (jd_tt, f64::INFINITY);
    for hour in -20..=20 {
        let t = jd_tt + f64::from(hour) / 24.0;
        let g = gamma(t)?;
        if g < best.1 {
            best = (t, g);
        }
    }
    golden_min(best.0 - 1.0 / 24.0, best.0 + 1.0 / 24.0, 1e-3, gamma)
}

/// The point of the Earth's day side nearest the shadow axis, and its distance on the
/// fundamental plane: zero when the axis meets the Earth.
pub(crate) struct DaySide {
    pub(crate) point: [f64; 3],
    pub(crate) distance: f64,
}

pub(crate) fn nearest_day_side(e: &Elements) -> DaySide {
    if let Some(point) = e.surface(e.x, e.y) {
        return DaySide {
            point,
            distance: 0.0,
        };
    }
    // The outline is nearly a circle, so the nearest point lies near the axis's direction.
    let toward = e.y.atan2(e.x);
    let theta = golden_min_infallible(toward - 0.5, toward + 0.5, |theta| {
        let [xi, eta, _] = e.project(e.outline(theta));
        (xi - e.x).hypot(eta - e.y)
    });
    let point = e.outline(theta);
    let [xi, eta, _] = e.project(point);
    DaySide {
        point,
        distance: (xi - e.x).hypot(eta - e.y),
    }
}

/// Total, annular or hybrid, from the sign of L2 along the central line, including its
/// ends at sunrise and sunset, where a hybrid eclipse's annular part often lies.
fn central_kind(track: &Track) -> Result<SolarKind, EclipseError> {
    let (start, end) = track.window();
    let step = 30.0 / time::SECONDS_PER_DAY;
    let on_earth = |t: f64| -> Result<Option<Shadow>, EclipseError> {
        let e = Elements::new(&track.at(t)?);
        Ok(e.surface(e.x, e.y).map(|p| e.shadow(p)))
    };
    let (mut total, mut annular) = (false, false);
    let mut note = |shadow: &Shadow| {
        if shadow.l2 < 0.0 {
            total = true
        } else {
            annular = true
        }
    };
    let mut previous: Option<(f64, bool)> = None;
    let mut t = start;
    while t <= end {
        let shadow = on_earth(t)?;
        if let Some(shadow) = &shadow {
            note(shadow);
        }
        if let Some((before, was)) = previous
            && was != shadow.is_some()
        {
            // Where the axis meets the Earth's edge, found to a millisecond.
            let (mut a, mut b) = (before, t);
            for _ in 0..30 {
                let mid = 0.5 * (a + b);
                if on_earth(mid)?.is_some() == was {
                    a = mid
                } else {
                    b = mid
                }
            }
            if let Some(edge) = on_earth(if was { a } else { b })? {
                note(&edge);
            }
        }
        previous = Some((t, shadow.is_some()));
        t += step;
    }
    Ok(match (total, annular) {
        (true, true) => SolarKind::Hybrid,
        (false, true) => SolarKind::Annular,
        _ => SolarKind::Total,
    })
}

/// The minimum of a unimodal `f` on `[a, b]`, to within `tolerance`.
pub(crate) fn golden_min<E>(
    mut a: f64,
    mut b: f64,
    tolerance: f64,
    f: impl Fn(f64) -> Result<f64, E>,
) -> Result<f64, E> {
    const R: f64 = 0.618_033_988_749_895;
    let mut c = b - R * (b - a);
    let mut d = a + R * (b - a);
    let (mut fc, mut fd) = (f(c)?, f(d)?);
    while b - a > tolerance {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - R * (b - a);
            fc = f(c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + R * (b - a);
            fd = f(d)?;
        }
    }
    Ok(0.5 * (a + b))
}

fn golden_min_infallible(a: f64, b: f64, f: impl Fn(f64) -> f64) -> f64 {
    match golden_min(a, b, 1e-9, |x| Ok::<f64, std::convert::Infallible>(f(x))) {
        Ok(x) => x,
        Err(never) => match never {},
    }
}

#[cfg(test)]
mod tests;
