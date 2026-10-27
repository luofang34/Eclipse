//! Lunation and Saros numbers, as the NASA catalogues count them.
//!
//! Lunation 0 is the new moon of 2000 January 6. Eclipses a Saros (223 lunations) apart
//! share a series; eclipses an inex (358 lunations) apart are in neighbouring series, the
//! later one numbered one lower. Each series is anchored on one catalogued eclipse.

/// The mean new moon of lunation 0, TT Julian date (Meeus, chapter 49).
pub const LUNATION_ZERO: f64 = 2_451_550.097_66;
/// The mean synodic month in days.
pub const SYNODIC_MONTH: f64 = 29.530_588_861;

/// The lunation of a solar eclipse at `jd_tt`: the nearest new moon's number.
pub fn solar_lunation(jd_tt: f64) -> i64 {
    ((jd_tt - LUNATION_ZERO) / SYNODIC_MONTH).round() as i64
}

/// The lunation of a lunar eclipse at `jd_tt`: the number of the new moon before it.
pub fn lunar_lunation(jd_tt: f64) -> i64 {
    ((jd_tt - LUNATION_ZERO) / SYNODIC_MONTH).floor() as i64
}

/// The Saros series of the solar eclipse of `lunation`; the 2017 August 21 eclipse,
/// lunation 218, is in series 145.
pub fn solar_saros(lunation: i64) -> i64 {
    series(lunation, 218, 145)
}

/// The Saros series of the lunar eclipse of `lunation`; the 2026 March 3 eclipse, lunation
/// 323, is in series 133.
pub fn lunar_saros(lunation: i64) -> i64 {
    series(lunation, 323, 133)
}

/// The series `s = anchor_series + m` with `lunation - anchor = 223 n + 358 m`, taking
/// the `m` that keeps the series within half a Saros cycle of the anchor's.
fn series(lunation: i64, anchor: i64, anchor_series: i64) -> i64 {
    // 358 = 135 mod 223, and 135 * 38 = 5130 = 1 mod 223.
    let m = ((lunation - anchor) * 38).rem_euclid(223);
    let m = if m > 111 { m - 223 } else { m };
    anchor_series + m
}

#[cfg(test)]
mod tests;
