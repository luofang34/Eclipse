//! The eclipse on show and what is drawn for it, computed once when it is chosen.

use eclipse_core::{
    Eclipse, EclipseError, LocalSolar, LunarEclipse, ShadowOutline, SolarEclipse, SolarMap,
    geodesy::LatLon,
};

/// The grid the lunar visibility is shaded on, degrees.
pub(crate) const VISIBILITY_STEP: f64 = 3.0;

/// A chosen eclipse with its map.
pub(crate) enum Selected {
    /// A solar eclipse and its curves.
    Solar {
        eclipse: Box<SolarEclipse>,
        map: Box<SolarMap>,
    },
    /// A lunar eclipse and where it is seen from.
    Lunar {
        eclipse: Box<LunarEclipse>,
        visibility: Visibility,
    },
}

/// The share of a lunar eclipse seen from each point of a grid.
pub(crate) struct Visibility {
    columns: usize,
    shares: Vec<f32>,
}

/// What a clicked place sees.
pub(crate) enum Local {
    /// A solar eclipse from there, or none.
    Solar(LatLon, Option<LocalSolar>),
    /// The Moon's altitude there at each contact of a lunar eclipse.
    Lunar(LatLon, Vec<(&'static str, f64, f64)>),
}

impl Selected {
    /// Computes the map of `eclipse`.
    pub(crate) fn of(eclipse: &Eclipse) -> Result<Self, EclipseError> {
        Ok(match eclipse {
            Eclipse::Solar(e) => Self::Solar {
                map: Box::new(e.map()?),
                eclipse: Box::new(e.clone()),
            },
            Eclipse::Lunar(e) => Self::Lunar {
                visibility: Visibility::of(e)?,
                eclipse: Box::new(e.clone()),
            },
        })
    }

    /// Greatest eclipse, TT Julian date.
    pub(crate) fn greatest_tt(&self) -> f64 {
        match self {
            Self::Solar { eclipse, .. } => eclipse.greatest_tt,
            Self::Lunar { eclipse, .. } => eclipse.greatest_tt,
        }
    }

    /// The span the time slider runs over: the eclipse from first to last contact, with a
    /// margin.
    pub(crate) fn span(&self) -> (f64, f64) {
        let margin = 10.0 / 1440.0;
        match self {
            Self::Solar { eclipse, .. } => {
                let (start, end) = eclipse.span();
                (start + margin, end - margin)
            }
            Self::Lunar { eclipse, .. } => {
                let c = eclipse.contacts;
                let g = eclipse.greatest_tt;
                (c.p1.unwrap_or(g) - margin, c.p4.unwrap_or(g) + margin)
            }
        }
    }

    /// Where the globe first looks.
    pub(crate) fn focus(&self) -> LatLon {
        match self {
            Self::Solar { eclipse, .. } => eclipse.greatest,
            // Half a hemisphere west of the Moon, so the globe shows both where the whole
            // eclipse is seen and where the Moon sets or rises during it.
            Self::Lunar { eclipse, .. } => {
                let below = eclipse.sublunar(eclipse.greatest_tt).unwrap_or_default();
                crate::overlay::destination(below, 270.0, 50.0)
            }
        }
    }

    /// The shadow on the ground at `jd_tt`, for a solar eclipse.
    pub(crate) fn shadow_at(&self, jd_tt: f64) -> Option<ShadowOutline> {
        match self {
            Self::Solar { eclipse, .. } => eclipse.shadow_outline(jd_tt).ok(),
            Self::Lunar { .. } => None,
        }
    }

    /// What `at` sees.
    pub(crate) fn local(&self, at: LatLon) -> Result<Local, EclipseError> {
        Ok(match self {
            Self::Solar { eclipse, .. } => Local::Solar(at, eclipse.local(at)?),
            Self::Lunar { eclipse, .. } => {
                let c = eclipse.contacts;
                let mut rows = Vec::new();
                for (name, tt) in [
                    ("P1", c.p1),
                    ("U1", c.u1),
                    ("U2", c.u2),
                    ("Greatest", Some(eclipse.greatest_tt)),
                    ("U3", c.u3),
                    ("U4", c.u4),
                    ("P4", c.p4),
                ] {
                    if let Some(tt) = tt {
                        rows.push((name, tt, eclipse.moon_altitude(at, tt)?));
                    }
                }
                Local::Lunar(at, rows)
            }
        })
    }
}

impl Visibility {
    fn of(eclipse: &LunarEclipse) -> Result<Self, EclipseError> {
        let columns = (360.0 / VISIBILITY_STEP).round() as usize;
        let rows = (180.0 / VISIBILITY_STEP).round() as usize;
        let mut shares = Vec::with_capacity((rows + 1) * (columns + 1));
        for row in 0..=rows {
            for column in 0..=columns {
                let at = LatLon {
                    lat: -90.0 + row as f64 * VISIBILITY_STEP,
                    lon: -180.0 + column as f64 * VISIBILITY_STEP,
                };
                shares.push(eclipse.visible_share(at)? as f32);
            }
        }
        Ok(Self { columns, shares })
    }

    /// The share of the eclipse seen at the grid point `at`.
    pub(crate) fn share(&self, at: LatLon) -> f32 {
        let row = ((at.lat + 90.0) / VISIBILITY_STEP).round() as usize;
        let column = ((at.lon + 180.0) / VISIBILITY_STEP).round() as usize;
        self.shares
            .get(row * (self.columns + 1) + column)
            .copied()
            .unwrap_or(0.0)
    }
}
