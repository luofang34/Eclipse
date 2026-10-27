//! The Canon fixtures: their dates, which may fall before year 0, and the years this build
//! of Navigate covers.

use eclipse_core::time::CalendarTime;

/// The fixture rows of `csv` whose eclipse falls in a year this build covers.
#[cfg(test)]
pub fn rows(csv: &str) -> impl Iterator<Item = &str> {
    csv.lines()
        .skip(1)
        .filter(|line| covered(year(line.split(',').next().unwrap_or_default())))
}

/// Whether Navigate's Sun and Moon cover `year` in this build.
#[cfg(test)]
pub fn covered(year: i32) -> bool {
    if cfg!(feature = "long-range") {
        (-1999..=2999).contains(&year)
    } else {
        (1800..=2199).contains(&year)
    }
}

/// The year of a `YYYY-MM-DD` or `-YYYY-MM-DD` date.
#[cfg(test)]
pub fn year(date: &str) -> i32 {
    let (sign, rest) = match date.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, date),
    };
    sign * rest[..4].parse::<i32>().unwrap()
}

/// The TT Julian date of a catalogue date and `HH:MM[:SS]` time.
#[cfg(test)]
pub fn jd(date: &str, time: &str) -> f64 {
    let digits = date.trim_start_matches('-');
    let month: u8 = digits[5..7].parse().unwrap();
    let day: u8 = digits[8..10].parse().unwrap();
    let t: Vec<f64> = time.split(':').map(|x| x.parse().unwrap()).collect();
    let hours = t[0] + t[1] / 60.0 + t.get(2).copied().unwrap_or(0.0) / 3600.0;
    CalendarTime::jd_at_midnight(year(date), month, day) + hours / 24.0
}
