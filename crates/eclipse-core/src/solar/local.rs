//! Local circumstances: how the eclipse looks from one place.

use super::{SolarEclipse, golden_min};
use crate::{
    EclipseError,
    besselian::Shadow,
    geodesy::{LatLon, altitude, surface},
};

/// The time step contacts are bracketed with, days.
const SCAN: f64 = 120.0 / 86_400.0;

/// A contact: a moment the Moon's limb touches the Sun's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// TT Julian date.
    pub tt: f64,
    /// The Sun's altitude then, degrees; negative when below the horizon.
    pub sun_altitude: f64,
}

/// The eclipse as seen from one place, computed whether or not the Sun is up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalSolar {
    /// The place.
    pub at: LatLon,
    /// The partial phase begins.
    pub c1: Option<Contact>,
    /// Totality or annularity begins.
    pub c2: Option<Contact>,
    /// Maximum eclipse.
    pub max: Contact,
    /// Totality or annularity ends.
    pub c3: Option<Contact>,
    /// The partial phase ends.
    pub c4: Option<Contact>,
    /// The fraction of the Sun's diameter covered at maximum.
    pub magnitude: f64,
    /// The fraction of the Sun's disc covered at maximum.
    pub obscuration: f64,
    /// The duration of totality or annularity, seconds.
    pub central_duration_s: Option<f64>,
}

impl SolarEclipse {
    /// The eclipse seen from `at`, or `None` where no part of it is seen.
    pub fn local(&self, at: LatLon) -> Result<Option<LocalSolar>, EclipseError> {
        let p = surface(at);
        let (start, end) = self.span();
        let shadow = |t: f64| -> Result<Shadow, EclipseError> { Ok(self.elements(t)?.shadow(p)) };
        // The distance from the axis has one minimum while the shadow passes.
        let mut best = (start, f64::INFINITY);
        let mut t = start;
        while t <= end {
            let d = shadow(t)?.delta;
            if d < best.1 {
                best = (t, d);
            }
            t += SCAN;
        }
        let lo = (best.0 - SCAN).max(start);
        let hi = (best.0 + SCAN).min(end);
        let max_tt = golden_min(lo, hi, 1e-8, |t| {
            Ok::<_, EclipseError>(-shadow(t)?.magnitude())
        })?;
        let at_max = shadow(max_tt)?;
        if at_max.magnitude() <= 0.0 {
            return Ok(None);
        }
        let contact = |tt: f64| -> Result<Contact, EclipseError> {
            Ok(Contact {
                tt,
                sun_altitude: altitude(at, self.elements(tt)?.sun),
            })
        };
        let outer = |s: &Shadow| s.delta - s.l1;
        let inner = |s: &Shadow| s.delta - s.l2.abs();
        let c1 = self.crossing(start, max_tt, &shadow, &outer)?;
        let c4 = self.crossing(end, max_tt, &shadow, &outer)?;
        let (c2, c3) = if at_max.central() {
            (
                self.crossing(start, max_tt, &shadow, &inner)?,
                self.crossing(end, max_tt, &shadow, &inner)?,
            )
        } else {
            (None, None)
        };
        Ok(Some(LocalSolar {
            at,
            c1: c1.map(contact).transpose()?,
            c2: c2.map(contact).transpose()?,
            max: contact(max_tt)?,
            c3: c3.map(contact).transpose()?,
            c4: c4.map(contact).transpose()?,
            magnitude: at_max.magnitude(),
            obscuration: at_max.obscuration(),
            central_duration_s: c2
                .zip(c3)
                .map(|(a, b)| (b - a) * crate::time::SECONDS_PER_DAY),
        }))
    }

    /// The instant between `outside` and `inside` where `f` changes sign, `f` being
    /// positive outside; `None` when it is not positive at `outside`.
    fn crossing(
        &self,
        outside: f64,
        inside: f64,
        shadow: &impl Fn(f64) -> Result<Shadow, EclipseError>,
        f: &impl Fn(&Shadow) -> f64,
    ) -> Result<Option<f64>, EclipseError> {
        if f(&shadow(outside)?) <= 0.0 || f(&shadow(inside)?) > 0.0 {
            return Ok(None);
        }
        let (mut a, mut b) = (outside, inside);
        for _ in 0..50 {
            let mid = 0.5 * (a + b);
            if f(&shadow(mid)?) > 0.0 {
                a = mid
            } else {
                b = mid
            }
        }
        Ok(Some(0.5 * (a + b)))
    }
}
