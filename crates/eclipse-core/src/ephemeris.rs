//! The geocentric apparent Sun and Moon in Earth-fixed axes, sampled from Navigate and
//! interpolated, so eclipse geometry costs vector arithmetic rather than an ephemeris call.
//!
//! Positions are in Earth equatorial radii. Navigate applies light time and aberration;
//! the samples are five minutes apart, and four-point Lagrange interpolation then agrees
//! with Navigate to a few metres, Earth's rotation of the axes included.

use navigate_celestial::{moon_ecef, sun_ecef};

use crate::{EclipseError, time::civil_epoch};

/// The WGS84 equatorial radius in metres, the unit of every position.
pub const EARTH_RADIUS_M: f64 = 6_378_137.0;
/// The WGS84 flattening.
pub const FLATTENING: f64 = 1.0 / 298.257_223_563;
/// Sample spacing, days.
const STEP: f64 = 5.0 / 1440.0;

/// Sun and Moon positions at one instant, Earth equatorial radii, ECEF axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bodies {
    /// The Sun's centre.
    pub sun: [f64; 3],
    /// The Moon's centre.
    pub moon: [f64; 3],
}

/// Sun and Moon positions sampled over a window.
#[derive(Clone, Debug)]
pub struct Track {
    start: f64,
    samples: Vec<Bodies>,
}

impl Track {
    /// Samples `[start, end]`, TT Julian dates, with a margin for interpolation.
    pub fn sample(start: f64, end: f64) -> Result<Self, EclipseError> {
        let first = start - 2.0 * STEP;
        let count = ((end - start) / STEP).ceil() as usize + 5;
        let samples = (0..count)
            .map(|i| bodies(first + i as f64 * STEP))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            start: first,
            samples,
        })
    }

    /// The first and last instants that interpolate with a full stencil.
    pub fn window(&self) -> (f64, f64) {
        let last = self.samples.len().saturating_sub(3) as f64;
        (self.start + STEP, self.start + last * STEP)
    }

    /// Sun and Moon at `jd_tt`.
    pub fn at(&self, jd_tt: f64) -> Result<Bodies, EclipseError> {
        let (start, end) = self.window();
        if !(start..=end).contains(&jd_tt) {
            return Err(EclipseError::OutsideWindow { jd_tt, start, end });
        }
        let position = (jd_tt - self.start) / STEP;
        let base = (position.floor() as usize).saturating_sub(1);
        let u = position - base as f64;
        let stencil = self
            .samples
            .get(base..base + 4)
            .ok_or(EclipseError::OutsideWindow { jd_tt, start, end })?;
        // Lagrange weights for nodes 0, 1, 2, 3 at u.
        let w = [
            -(u - 1.0) * (u - 2.0) * (u - 3.0) / 6.0,
            u * (u - 2.0) * (u - 3.0) / 2.0,
            -u * (u - 1.0) * (u - 3.0) / 2.0,
            u * (u - 1.0) * (u - 2.0) / 6.0,
        ];
        let mut out = Bodies {
            sun: [0.0; 3],
            moon: [0.0; 3],
        };
        for (weight, sample) in w.iter().zip(stencil) {
            for axis in 0..3 {
                out.sun[axis] += weight * sample.sun[axis];
                out.moon[axis] += weight * sample.moon[axis];
            }
        }
        Ok(out)
    }
}

/// Sun and Moon at `jd_tt` straight from Navigate.
pub fn bodies(jd_tt: f64) -> Result<Bodies, EclipseError> {
    let epoch = civil_epoch(jd_tt);
    let sun = sun_ecef(epoch, None).map_err(|source| EclipseError::Ephemeris {
        body: "Sun",
        jd_tt,
        source,
    })?;
    let moon = moon_ecef(epoch, None).map_err(|source| EclipseError::Ephemeris {
        body: "Moon",
        jd_tt,
        source,
    })?;
    let scale = |direction: [f64; 3], distance: f64| {
        direction.map(|component| component * distance / EARTH_RADIUS_M)
    };
    Ok(Bodies {
        sun: scale(sun.direction, sun.distance_m),
        moon: scale(moon.direction, moon.distance_m),
    })
}

#[cfg(test)]
mod tests;
