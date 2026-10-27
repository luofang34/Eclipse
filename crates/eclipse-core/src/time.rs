//! Time: eclipses are computed in TT, as Julian days, and shown in TT and UT.
//!
//! Navigate's Sun and Moon take the TT instant itself. Its star catalogue takes a civil
//! clock reading, UT1 before 1972 and UTC from 1972, which [`civil_epoch`] gives.

use navigate_celestial::{DeltaT, Epoch, TimeScale, delta_t};

/// Seconds in a day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
/// Julian date minus the modified Julian date.
const MJD_OFFSET: f64 = 2_400_000.5;

/// The TT instant at Julian date `jd_tt`.
pub fn tt_epoch(jd_tt: f64) -> Epoch {
    Epoch::from_mjd_in_time_scale(jd_tt - MJD_OFFSET, TimeScale::TT)
}

/// The civil reading Navigate interprets as the TT instant `jd_tt`.
pub fn civil_epoch(jd_tt: f64) -> Epoch {
    let epoch = tt_epoch(jd_tt);
    if epoch >= Epoch::from_gregorian_utc_at_midnight(1972, 1, 1) {
        return epoch;
    }
    // Before 1972 the reading is UT1, and Navigate takes Delta T at UT1.
    let mjd_tt = jd_tt - MJD_OFFSET;
    let mut ut1 = mjd_tt;
    for _ in 0..3 {
        let at_ut1 = Epoch::from_mjd_in_time_scale(ut1, TimeScale::TT);
        ut1 = mjd_tt - delta_t(at_ut1).seconds / SECONDS_PER_DAY;
    }
    Epoch::from_mjd_utc(ut1)
}

/// Delta T = TT - UT1 at the TT instant `jd_tt`, with its uncertainty and basis.
pub fn delta_t_at(jd_tt: f64) -> DeltaT {
    delta_t(tt_epoch(jd_tt))
}

/// UT1 as a Julian date for the TT instant `jd_tt`.
pub fn ut1(jd_tt: f64) -> f64 {
    jd_tt - delta_t_at(jd_tt).seconds / SECONDS_PER_DAY
}

/// A Gregorian calendar date and time of day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarTime {
    /// Year.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month.
    pub day: u8,
    /// Hour.
    pub hour: u8,
    /// Minute.
    pub minute: u8,
    /// Second, rounded.
    pub second: u8,
}

impl CalendarTime {
    /// The Gregorian date and time of Julian date `jd`, rounded to the second (Meeus,
    /// *Astronomical Algorithms*, chapter 7).
    pub fn from_jd(jd: f64) -> Self {
        let seconds = ((jd + 0.5).fract() * SECONDS_PER_DAY).round();
        let (days, seconds) = if seconds >= SECONDS_PER_DAY {
            ((jd + 0.5).floor() + 1.0, 0.0)
        } else {
            ((jd + 0.5).floor(), seconds)
        };
        let alpha = ((days - 1_867_216.25) / 36_524.25).floor();
        let a = days + 1.0 + alpha - (alpha / 4.0).floor();
        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();
        let day = b - d - (30.6001 * e).floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
        let s = seconds as u32;
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
            hour: (s / 3600) as u8,
            minute: (s / 60 % 60) as u8,
            second: (s % 60) as u8,
        }
    }

    /// The Julian date of a Gregorian date at 0h.
    pub fn jd_at_midnight(year: i32, month: u8, day: u8) -> f64 {
        let (y, m) = if month <= 2 {
            (f64::from(year) - 1.0, f64::from(month) + 12.0)
        } else {
            (f64::from(year), f64::from(month))
        };
        let a = (y / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + f64::from(day) + b
            - 1524.5
    }

    /// `YYYY Mon DD`, as the NASA catalogues write dates.
    pub fn date(&self) -> String {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month = MONTHS
            .get(usize::from(self.month).saturating_sub(1))
            .copied()
            .unwrap_or("?");
        format!("{} {month} {:02}", self.year, self.day)
    }

    /// `HH:MM:SS`.
    pub fn time(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    /// `YYYY-MM-DD`.
    pub fn iso_date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests;
