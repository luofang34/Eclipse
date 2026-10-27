//! Finding the eclipses of a year: one candidate of each kind per lunation.

use crate::{
    EclipseError, LunarEclipse, SolarEclipse,
    saros::{LUNATION_ZERO, SYNODIC_MONTH},
    time::CalendarTime,
};

/// A solar or lunar eclipse.
#[derive(Clone, Debug)]
pub enum Eclipse {
    /// An eclipse of the Sun.
    Solar(SolarEclipse),
    /// An eclipse of the Moon.
    Lunar(LunarEclipse),
}

impl Eclipse {
    /// Greatest eclipse, TT Julian date.
    pub fn greatest_tt(&self) -> f64 {
        match self {
            Self::Solar(e) => e.greatest_tt,
            Self::Lunar(e) => e.greatest_tt,
        }
    }
}

/// The eclipses whose greatest eclipse falls in `year` (by TT), in time order. Eclipses
/// beyond the range Navigate covers are left out.
pub fn eclipses_in_year(year: i32) -> Result<Vec<Eclipse>, EclipseError> {
    let start = CalendarTime::jd_at_midnight(year, 1, 1);
    let end = CalendarTime::jd_at_midnight(year + 1, 1, 1);
    let first = ((start - LUNATION_ZERO) / SYNODIC_MONTH).floor() as i64 - 1;
    let last = ((end - LUNATION_ZERO) / SYNODIC_MONTH).ceil() as i64;
    let mut found = Vec::new();
    for k in first..=last {
        let new_moon = LUNATION_ZERO + k as f64 * SYNODIC_MONTH;
        if let Some(e) = in_range(SolarEclipse::near(new_moon))? {
            found.push(Eclipse::Solar(e));
        }
        if let Some(e) = in_range(LunarEclipse::near(new_moon + 0.5 * SYNODIC_MONTH))? {
            found.push(Eclipse::Lunar(e));
        }
    }
    found.retain(|e| (start..end).contains(&e.greatest_tt()));
    found.sort_by(|a, b| a.greatest_tt().total_cmp(&b.greatest_tt()));
    Ok(found)
}

/// An eclipse, or none where Navigate's range ends.
fn in_range<T>(result: Result<Option<T>, EclipseError>) -> Result<Option<T>, EclipseError> {
    match result {
        Err(EclipseError::Ephemeris { .. }) => Ok(None),
        other => other,
    }
}

#[cfg(test)]
mod tests;
