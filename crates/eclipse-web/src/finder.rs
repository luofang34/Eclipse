//! The eclipses seen from one place: a search forward year by year, one year a frame so
//! the page stays live, until enough are found or the years run out.

use eclipse_core::{Eclipse, LunarKind, eclipses_in_year, geodesy::LatLon};

/// Sightings the search stops at.
const WANTED: usize = 8;
/// Years the search covers at most.
const YEARS: i32 = 60;

/// One eclipse seen from the place.
pub(crate) struct Sighting {
    pub(crate) eclipse: Eclipse,
    /// When it is greatest from there, TT Julian date.
    pub(crate) best_tt: f64,
    /// What the place sees.
    pub(crate) what: String,
}

/// A search under way or finished.
pub(crate) struct Finder {
    pub(crate) site: LatLon,
    pub(crate) next_year: i32,
    last_year: i32,
    pub(crate) found: Vec<Sighting>,
}

impl Finder {
    /// A search from `site` starting with `year`.
    pub(crate) fn new(site: LatLon, year: i32) -> Self {
        Self {
            site,
            next_year: year,
            last_year: (year + YEARS - 1).min(*crate::panel::YEARS.end()),
            found: Vec::new(),
        }
    }

    /// Whether the search has ended.
    pub(crate) fn done(&self) -> bool {
        self.found.len() >= WANTED || self.next_year > self.last_year
    }

    /// Searches one more year.
    pub(crate) fn step(&mut self) {
        if self.done() {
            return;
        }
        let year = self.next_year;
        self.next_year += 1;
        match eclipses_in_year(year) {
            Ok(list) => {
                let seen = list.into_iter().filter_map(|e| sighting(e, self.site));
                self.found.extend(seen);
                self.found.truncate(WANTED);
            }
            Err(error) => tracing::error!(%error, year, "eclipses unavailable"),
        }
    }
}

/// What `site` sees of `eclipse`, if any of it is above the horizon: a solar eclipse with
/// the Sun up at its maximum, or a partial or total lunar eclipse with the Moon up at
/// greatest eclipse.
fn sighting(eclipse: Eclipse, site: LatLon) -> Option<Sighting> {
    match &eclipse {
        Eclipse::Solar(e) => {
            let local = e.local(site).ok()??;
            if local.magnitude <= 0.0 || local.max.sun_altitude <= 0.0 {
                return None;
            }
            let what = match local.central_duration_s {
                Some(seconds) => {
                    let kind = if local.magnitude >= 1.0 {
                        "Total"
                    } else {
                        "Annular"
                    };
                    let s = seconds.round() as i64;
                    format!("{kind} solar, {}m{:02}s", s / 60, s % 60)
                }
                // Near 100 % a rounded share would read as total.
                None if local.obscuration > 0.99 => {
                    format!("Partial solar, {:.1}% covered", 100.0 * local.obscuration)
                }
                None => format!("Partial solar, {:.0}% covered", 100.0 * local.obscuration),
            };
            Some(Sighting {
                best_tt: local.max.tt,
                what,
                eclipse,
            })
        }
        Eclipse::Lunar(e) => {
            if e.kind == LunarKind::Penumbral {
                return None;
            }
            let altitude = e.moon_altitude(site, e.greatest_tt).ok()?;
            let (best_tt, what) = (
                e.greatest_tt,
                format!("{} lunar, Moon {altitude:.0}° up", e.kind.name()),
            );
            (altitude > 0.0).then_some(Sighting {
                best_tt,
                what,
                eclipse,
            })
        }
    }
}
