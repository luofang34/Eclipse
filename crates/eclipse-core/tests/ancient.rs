//! Eclipses recorded before the telescope, against what eclipse-core computes with
//! Navigate's long-range Sun and Moon.
//!
//! Before about 1600 Delta T is known only from records like these, so the Canon's model
//! of it is fitted to them, uncertain by minutes to hours. A record that an eclipse was
//! total somewhere is therefore checked as the claim it supports: that the place was in the
//! path for some Delta T within twice the model's uncertainty. The eclipse itself, in TT,
//! does not depend on Delta T; a larger Delta T only turns the Earth less under it.
#![cfg(feature = "long-range")]

use eclipse_core::{LunarEclipse, SolarEclipse, geodesy::LatLon, time::CalendarTime};

/// The Earth turns 15.041 arcseconds in a second.
const DEGREES_PER_SECOND: f64 = 15.041 / 3600.0;

/// The solar eclipse of a Julian-calendar date, found from its noon.
#[cfg(test)]
fn solar(year: i32, month: u8, day: u8) -> SolarEclipse {
    SolarEclipse::near(CalendarTime::jd_at_midnight(year, month, day) + 0.5)
        .unwrap()
        .unwrap()
}

/// The Delta T offsets, within twice the model's uncertainty, for which some place in
/// `sites` sees the eclipse as total or annular; the band searched is printed with them.
#[cfg(test)]
fn central_offsets(e: &SolarEclipse, sites: &[LatLon]) -> Vec<f64> {
    let sigma = e.delta_t.uncertainty_seconds;
    let steps = 80;
    let mut offsets = Vec::new();
    for i in 0..=steps {
        let offset = -2.0 * sigma + 4.0 * sigma * f64::from(i) / f64::from(steps);
        // A Delta T larger by `offset` puts the shadow further east, as if each place lay
        // that much further west under the model's Delta T.
        let central = sites.iter().any(|at| {
            let shifted = LatLon {
                lat: at.lat,
                lon: at.lon - offset * DEGREES_PER_SECOND,
            };
            e.local(shifted)
                .unwrap()
                .is_some_and(|seen| seen.central_duration_s.is_some())
        });
        if central {
            offsets.push(offset);
        }
    }
    println!(
        "Delta T {:.0} s ± {:.0} s: central for offsets {:?}..{:?} s",
        e.delta_t.seconds,
        sigma,
        offsets.first(),
        offsets.last()
    );
    offsets
}

/// The Assyrian eponym list, in the year of Bur-Sagale of Guzana, after a revolt in the city
/// of Assur: "In the month Simanu an eclipse of the sun took place" (the eclipse of -762
/// June 15, Julian). The record does not say total; it says a solar eclipse was seen in
/// Assyria. Nineveh, the capital, is checked for a deep partial phase at the model's
/// Delta T, and for lying in the path within the model's uncertainty.
#[test]
fn bur_sagale_in_assyria() {
    let e = solar(-762, 6, 15);
    let nineveh = LatLon {
        lat: 36.36,
        lon: 43.15,
    };
    let seen = e.local(nineveh).unwrap().unwrap();
    println!("-762 Nineveh: magnitude {:.3}", seen.magnitude);
    assert!(seen.magnitude > 0.9);
    assert!(!central_offsets(&e, &[nineveh]).is_empty());
}

/// Herodotus, Histories 1.74, on the war of the Lydians and the Medes that Thales foretold
/// the eclipse of: "just as the battle was growing warm, day was on a sudden changed into
/// night" (Rawlinson's translation). Herodotus names no battlefield; the Halys
/// (Kizilirmak), the border of the two kingdoms in central Anatolia, is a modern
/// placing. A stretch of that country is checked: the eclipse of -584 May 28 is total
/// somewhere in it for a Delta T within the model's uncertainty.
#[test]
fn thales_by_the_halys() {
    let e = solar(-584, 5, 28);
    let mut country = Vec::new();
    for i in 0..=6 {
        for j in 0..=6 {
            country.push(LatLon {
                lat: 38.5 + 0.5 * f64::from(i),
                lon: 32.5 + 0.5 * f64::from(j),
            });
        }
    }
    assert!(!central_offsets(&e, &country).is_empty());
}

/// The Babylonian tablets of Seleucid year 175 on the solar eclipse of -135 April 15:
/// "[Ven]us, Mercury and the Normal Stars were visible; Jupiter and Mars, which were in
/// their period of invisibility, were visible in its eclipse" (as translated in the Journal
/// for Occultation Astronomy 2023-2, after Stephenson and Steele, JHA 37, 2006). Stars by
/// day mean totality at Babylon: Babylon is in the path for a Delta T within the model's
/// uncertainty.
#[test]
fn babylon_in_totality() {
    let e = solar(-135, 4, 15);
    let babylon = LatLon {
        lat: 32.54,
        lon: 44.42,
    };
    assert!(!central_offsets(&e, &[babylon]).is_empty());
}

/// Halley at the Royal Society's house in Crane Court, London, 22 April 1715 Old Style
/// (3 May Gregorian): the Sun was "totally obscured for 3',23'' of Time" (Phil. Trans. 29,
/// 1714-1716). Delta T is then known to seconds, so the duration is compared as the 1851
/// and 1919 timings are.
#[test]
fn halley_in_london() {
    let e = SolarEclipse::near(CalendarTime::jd_at_midnight(1715, 5, 3) + 0.4)
        .unwrap()
        .unwrap();
    let crane_court = LatLon {
        lat: 51.5146,
        lon: -0.1082,
    };
    let seen = e.local(crane_court).unwrap().unwrap();
    let computed = seen.central_duration_s.unwrap();
    println!("1715 London: computed {computed:.1} s, Halley 203 s");
    assert!((computed - 203.0).abs() <= 5.0);
}

/// Columbus at Santa Gloria (St Ann's Bay), Jamaica, on the evening of 29 February 1504
/// (1 March UT): he timed the eclipse by his hourglass, and "the totality ended after 48
/// minutes" (The Guardian, 15 July 1999, after his account). The total phase was above his
/// horizon, and lasted 48 minutes to the minute of an hourglass.
#[test]
fn columbus_in_jamaica() {
    let e = LunarEclipse::near(CalendarTime::jd_at_midnight(1504, 3, 1) + 0.03)
        .unwrap()
        .unwrap();
    let santa_gloria = LatLon {
        lat: 18.44,
        lon: -77.20,
    };
    let (u2, u3) = (e.contacts.u2.unwrap(), e.contacts.u3.unwrap());
    let minutes = (u3 - u2) * 1440.0;
    let rising = e.moon_altitude(santa_gloria, u2).unwrap();
    let after = e.moon_altitude(santa_gloria, u3).unwrap();
    println!("1504 Jamaica: totality {minutes:.1} min, Moon {rising:.1}° to {after:.1}° up");
    assert!(rising > 0.0 && after > 0.0);
    assert!((minutes - 48.0).abs() <= 1.5);
}
