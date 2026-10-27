//! A lunar eclipse: the Moon's passage through the Earth's shadow, with the shadow's
//! radii enlarged as Danjon proposed, as the NASA Five Millennium Canon computes them.

use navigate_celestial::DeltaT;

use crate::{
    EclipseError, Track,
    besselian::{K_PENUMBRA, SUN_RADIUS},
    geodesy::{LatLon, cross, dot, norm, scale, sub, surface, unit, up},
    saros,
    solar::golden_min,
    time,
};

/// Half the span sampled around greatest eclipse, days.
const HALF_SPAN: f64 = 4.5 / 24.0;
/// The Earth's rotation, radians per day of TT, for directions among the stars.
const SIDEREAL_RATE: f64 = std::f64::consts::TAU * 1.002_737_909_35;
/// Danjon's enlargement of the Earth's radius, 1/85, less 1/594 for its flattening.
const DANJON: f64 = 1.01;

/// What kind of lunar eclipse it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarKind {
    /// The Moon enters only the penumbra.
    Penumbral,
    /// Part of the Moon enters the umbra.
    Partial,
    /// The whole Moon enters the umbra.
    Total,
}

impl LunarKind {
    /// The Canon's one-letter code.
    pub fn code(self) -> char {
        match self {
            Self::Penumbral => 'N',
            Self::Partial => 'P',
            Self::Total => 'T',
        }
    }

    /// The name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Penumbral => "Penumbral",
            Self::Partial => "Partial",
            Self::Total => "Total",
        }
    }
}

/// The contacts of a lunar eclipse, TT Julian dates; phases that do not occur are `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LunarContacts {
    /// The Moon enters the penumbra.
    pub p1: Option<f64>,
    /// The Moon enters the umbra.
    pub u1: Option<f64>,
    /// Totality begins.
    pub u2: Option<f64>,
    /// Totality ends.
    pub u3: Option<f64>,
    /// The Moon leaves the umbra.
    pub u4: Option<f64>,
    /// The Moon leaves the penumbra.
    pub p4: Option<f64>,
}

/// The Moon against the shadow at one instant, seen from the Earth's centre, in
/// arcminutes on the sky about the shadow's centre: east and north.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoonInShadow {
    /// Toward increasing right ascension.
    pub east: f64,
    /// Toward the celestial north pole.
    pub north: f64,
}

/// A lunar eclipse.
#[derive(Clone, Debug)]
pub struct LunarEclipse {
    track: Track,
    /// Greatest eclipse, when the Moon passes closest to the shadow's axis, TT Julian date.
    pub greatest_tt: f64,
    /// Delta T at greatest eclipse.
    pub delta_t: DeltaT,
    /// Its type.
    pub kind: LunarKind,
    /// The Moon's least distance from the shadow axis in Earth equatorial radii, positive
    /// when it passes north of it.
    pub gamma: f64,
    /// The fraction of the Moon's diameter in the penumbra at greatest eclipse.
    pub penumbral_magnitude: f64,
    /// The fraction of the Moon's diameter in the umbra at greatest eclipse.
    pub umbral_magnitude: f64,
    /// The contacts.
    pub contacts: LunarContacts,
    /// The Canon's lunation number.
    pub lunation: i64,
    /// The Saros series.
    pub saros: i64,
}

/// The shadow and the Moon at one instant, radians.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    separation: f64,
    moon: f64,
    umbra: f64,
    penumbra: f64,
    parallax: f64,
}

impl LunarEclipse {
    /// The lunar eclipse at the full moon nearest `jd_tt`, or `None` if there is none.
    pub fn near(jd_tt: f64) -> Result<Option<Self>, EclipseError> {
        let separation =
            |t: f64| -> Result<f64, EclipseError> { Ok(geometry(&crate::bodies(t)?).separation) };
        let mut best = (jd_tt, f64::INFINITY);
        for hour in -20..=20 {
            let t = jd_tt + f64::from(hour) / 24.0;
            let s = separation(t)?;
            if s < best.1 {
                best = (t, s);
            }
        }
        let coarse = golden_min(best.0 - 1.0 / 24.0, best.0 + 1.0 / 24.0, 1e-3, separation)?;
        let g = geometry(&crate::bodies(coarse)?);
        if g.separation > g.penumbra + g.moon + 0.01 {
            return Ok(None);
        }
        let track = Track::sample(coarse - HALF_SPAN, coarse + HALF_SPAN)?;
        let at = |t: f64| -> Result<Geometry, EclipseError> { Ok(geometry(&track.at(t)?)) };
        let greatest_tt = golden_min(coarse - 0.05, coarse + 0.05, 1e-7, |t| {
            Ok::<_, EclipseError>(at(t)?.separation)
        })?;
        let g = at(greatest_tt)?;
        let magnitude = |radius: f64| (radius + g.moon - g.separation) / (2.0 * g.moon);
        let (penumbral_magnitude, umbral_magnitude) = (magnitude(g.penumbra), magnitude(g.umbra));
        let kind = if umbral_magnitude >= 1.0 {
            LunarKind::Total
        } else if umbral_magnitude > 0.0 {
            LunarKind::Partial
        } else if penumbral_magnitude > 0.0 {
            LunarKind::Penumbral
        } else {
            return Ok(None);
        };
        let lunation = saros::lunar_lunation(greatest_tt);
        let mut eclipse = Self {
            track,
            greatest_tt,
            delta_t: time::delta_t_at(greatest_tt),
            kind,
            gamma: 0.0,
            penumbral_magnitude,
            umbral_magnitude,
            contacts: LunarContacts::default(),
            lunation,
            saros: saros::lunar_saros(lunation),
        };
        let offset = eclipse.moon_in_shadow(greatest_tt)?;
        eclipse.gamma = g.separation / g.parallax * offset.north.signum();
        eclipse.contacts = eclipse.find_contacts()?;
        Ok(Some(eclipse))
    }

    /// The sampled span, TT Julian dates.
    pub fn span(&self) -> (f64, f64) {
        self.track.window()
    }

    /// The radii of the penumbra, the umbra and the Moon at `jd_tt`, arcminutes.
    pub fn radii(&self, jd_tt: f64) -> Result<[f64; 3], EclipseError> {
        let g = geometry(&self.track.at(jd_tt)?);
        Ok([g.penumbra, g.umbra, g.moon].map(|r| r.to_degrees() * 60.0))
    }

    /// Where the Moon's centre stands against the shadow at `jd_tt`.
    pub fn moon_in_shadow(&self, jd_tt: f64) -> Result<MoonInShadow, EclipseError> {
        let bodies = self.track.at(jd_tt)?;
        let anti = scale(unit(bodies.sun), -1.0);
        let (east, north) = sky_axes(anti);
        let moon = unit(bodies.moon);
        Ok(MoonInShadow {
            east: dot(moon, east).asin().to_degrees() * 60.0,
            north: dot(moon, north).asin().to_degrees() * 60.0,
        })
    }

    /// The direction the shadow moves among the stars, the ecliptic's, as a unit vector
    /// (east, north) on the sky.
    pub fn shadow_motion(&self) -> Result<[f64; 2], EclipseError> {
        let h = 1.0 / 24.0;
        let at = |t: f64| -> Result<[f64; 3], EclipseError> {
            let anti = scale(unit(self.track.at(t)?.sun), -1.0);
            // Undo the Earth's turn since greatest eclipse, so the directions are among
            // the stars.
            let angle = SIDEREAL_RATE * (t - self.greatest_tt);
            let (s, c) = angle.sin_cos();
            Ok([
                c * anti[0] - s * anti[1],
                s * anti[0] + c * anti[1],
                anti[2],
            ])
        };
        let motion = sub(at(self.greatest_tt + h)?, at(self.greatest_tt - h)?);
        let (east, north) = sky_axes(at(self.greatest_tt)?);
        let (x, y) = (dot(motion, east), dot(motion, north));
        let length = x.hypot(y);
        Ok([x / length, y / length])
    }

    /// The Moon's altitude at `at`, degrees, its parallax included.
    pub fn moon_altitude(&self, at: LatLon, jd_tt: f64) -> Result<f64, EclipseError> {
        let moon = self.track.at(jd_tt)?.moon;
        let d = sub(moon, surface(at));
        Ok((dot(d, up(at)) / norm(d))
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees())
    }

    /// Where the Moon stands overhead at `jd_tt`.
    pub fn sublunar(&self, jd_tt: f64) -> Result<LatLon, EclipseError> {
        let moon = unit(self.track.at(jd_tt)?.moon);
        Ok(LatLon {
            lat: moon[2].asin().to_degrees(),
            lon: moon[1].atan2(moon[0]).to_degrees(),
        })
    }

    /// The share of the eclipse, from the Moon entering the penumbra to leaving it, during
    /// which the Moon is above the horizon at `at`.
    pub fn visible_share(&self, at: LatLon) -> Result<f64, EclipseError> {
        let (Some(start), Some(end)) = (self.contacts.p1, self.contacts.p4) else {
            return Ok(0.0);
        };
        const SAMPLES: u32 = 24;
        let mut up_count = 0;
        for i in 0..=SAMPLES {
            let t = start + (end - start) * f64::from(i) / f64::from(SAMPLES);
            if self.moon_altitude(at, t)? > 0.0 {
                up_count += 1;
            }
        }
        Ok(f64::from(up_count) / f64::from(SAMPLES + 1))
    }

    fn find_contacts(&self) -> Result<LunarContacts, EclipseError> {
        let (start, end) = self.span();
        let edge = |radius: fn(&Geometry) -> f64, from: f64| -> Result<Option<f64>, EclipseError> {
            let outside = |t: f64| -> Result<f64, EclipseError> {
                let g = geometry(&self.track.at(t)?);
                Ok(g.separation - radius(&g))
            };
            if outside(self.greatest_tt)? >= 0.0 || outside(from)? <= 0.0 {
                return Ok(None);
            }
            let (mut a, mut b) = (from, self.greatest_tt);
            for _ in 0..50 {
                let mid = 0.5 * (a + b);
                if outside(mid)? > 0.0 {
                    a = mid
                } else {
                    b = mid
                }
            }
            Ok(Some(0.5 * (a + b)))
        };
        let penumbra = |g: &Geometry| g.penumbra + g.moon;
        let partial = |g: &Geometry| g.umbra + g.moon;
        let total = |g: &Geometry| g.umbra - g.moon;
        Ok(LunarContacts {
            p1: edge(penumbra, start)?,
            u1: edge(partial, start)?,
            u2: edge(total, start)?,
            u3: edge(total, end)?,
            u4: edge(partial, end)?,
            p4: edge(penumbra, end)?,
        })
    }
}

/// Unit vectors east and north on the sky at the direction `toward`.
fn sky_axes(toward: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let east = unit(cross([0.0, 0.0, 1.0], toward));
    (east, cross(toward, east))
}

fn geometry(bodies: &crate::Bodies) -> Geometry {
    let (sun, moon) = (norm(bodies.sun), norm(bodies.moon));
    let anti = scale(bodies.sun, -1.0 / sun);
    let separation = dot(anti, scale(bodies.moon, 1.0 / moon))
        .clamp(-1.0, 1.0)
        .acos();
    let parallax = (1.0 / moon).asin();
    let solar_parallax = (1.0 / sun).asin();
    let sun_radius = (SUN_RADIUS / sun).asin();
    Geometry {
        separation,
        moon: (K_PENUMBRA / moon).asin(),
        umbra: DANJON * parallax - sun_radius + solar_parallax,
        penumbra: DANJON * parallax + sun_radius + solar_parallax,
        parallax,
    }
}

#[cfg(test)]
mod tests;
