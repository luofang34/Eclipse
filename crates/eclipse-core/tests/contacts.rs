//! The beginning and end of totality as nineteenth- and twentieth-century observers timed
//! them, against the contacts computed at their stations with Navigate's Delta T.
//!
//! In these years Delta T is the spline Stephenson, Morrison and Hohenkerk fitted to
//! thousands of occultations; the USNO's table differs from it by up to six seconds before
//! 1900. A contact moves by about a second for each second of Delta T, so one timing cannot
//! choose between them, but many independent ones would show a bias.
//!
//! Times are as published, reduced to Greenwich by the longitude or clock correction the
//! observer gave, and to the civil day where they were reckoned from noon.
//!
//! Only the long-range Moon, fitted to DE431 within half an arcsecond, is checked: the
//! default series is bounded at 18 arcseconds, which can move a contact by ten seconds.
#![cfg(feature = "long-range")]

use eclipse_core::{SolarEclipse, geodesy::LatLon, time::CalendarTime};

/// A timed contact: the eclipse's civil date, the station, and the contacts' Greenwich times
/// in seconds of the day, the end of totality when it was timed.
struct Timed {
    date: (i32, u8, u8),
    station: &'static str,
    at: LatLon,
    second: f64,
    third: Option<f64>,
}

/// Seconds of the day of `h`:`m`:`s`.
const fn hms(h: f64, m: f64, s: f64) -> f64 {
    h * 3600.0 + m * 60.0 + s
}

/// Christiania mean time to Greenwich: the observatory's longitude, 0h 42m 53.9s east.
const CHRISTIANIA: f64 = hms(0.0, 42.0, 53.9);
/// Göteborg mean time to Greenwich: the Navigation School's longitude, 0h 47m 50.6s east.
const GOTEBORG: f64 = hms(0.0, 47.0, 50.6);
/// Denver local mean time to Greenwich: Penrose's longitude, 105° 1′ 43″ west.
const DENVER: f64 = (105.0 + 1.0 / 60.0 + 43.0 / 3600.0) * 240.0;

/// "Solar Eclipse of July 28, 1851", MNRAS 12, 41 (1852), and the reports cited per row.
const TIMED: [Timed; 8] = [
    // Dunkin, near the observatory: "Beginning of Totality ... 3 44 49·5 by Chronometer.
    // End ... 3 47 24·0"; "The chronometer was 1m 58s·2 slow on Christiania mean time" (p. 45).
    Timed {
        date: (1851, 7, 28),
        station: "Christiania, Dunkin",
        at: LatLon {
            lat: 59.9014,
            lon: 10.7246,
        },
        second: hms(15.0, 46.0, 47.7) - CHRISTIANIA,
        third: Some(hms(15.0, 49.0, 22.2) - CHRISTIANIA),
    },
    // Snow: "Beginning of Totality 3 46 53 Christiania M.T. End — — 3 49 28" (p. 47).
    Timed {
        date: (1851, 7, 28),
        station: "Christiania, Snow",
        at: LatLon {
            lat: 59.9014,
            lon: 10.7246,
        },
        second: hms(15.0, 46.0, 53.0) - CHRISTIANIA,
        third: Some(hms(15.0, 49.0, 28.0) - CHRISTIANIA),
    },
    // Pettersson at the Navigation School: "Beginning of Totality 3 55 52·2 Göteborg M.T.",
    // "End of Totality ... 3 59 8·2 Göteborg M.T." (pp. 58-59).
    Timed {
        date: (1851, 7, 28),
        station: "Göteborg, Pettersson",
        at: LatLon {
            lat: 57.70172,
            lon: 11.96083,
        },
        second: hms(15.0, 55.0, 52.2) - GOTEBORG,
        third: Some(hms(15.0, 59.0, 8.2) - GOTEBORG),
    },
    // Airy at Hvalås: totality "at 3h 7m 50s" and its end "3 11 6 by the watch";
    // "Chronometer slow on Göttenburg M.T. 0h 48m 6s·55" (pp. 60-61).
    Timed {
        date: (1851, 7, 28),
        station: "Hvalås, Airy",
        at: LatLon {
            lat: 57.70294,
            lon: 12.00167,
        },
        second: hms(15.0, 55.0, 56.55) - GOTEBORG,
        third: Some(hms(15.0, 59.0, 12.55) - GOTEBORG),
    },
    // Lassell at Trollhättan: "Total Obscuration 3 13 6·8 Watch ... Reappearance ...
    // 3 16 26·0"; "The watch ... was fast on Greenwich 5m 57s" (p. 52).
    Timed {
        date: (1851, 7, 28),
        station: "Trollhättan, Lassell",
        at: LatLon {
            lat: 58.29167,
            lon: 12.28333,
        },
        second: hms(15.0, 7.0, 9.8),
        third: Some(hms(15.0, 10.0, 29.0)),
    },
    // Penrose outside Denver, in local mean time: "Second contact 3 28 51·5 / Third contact
    // 3 31 35·6" (MNRAS 39, 48, 1878).
    Timed {
        date: (1878, 7, 29),
        station: "Denver, Penrose",
        at: LatLon {
            lat: 39.77472,
            lon: -105.02861,
        },
        second: hms(15.0, 28.0, 51.5) + DENVER,
        third: Some(hms(15.0, 31.0, 35.6) + DENVER),
    },
    // Maunder at Algiers, watches compared with the observatory's clock: "2nd 4 17 17·1"
    // Greenwich mean time (MNRAS 60, 588, 1900). The third contact printed with it is the
    // second plus the party's duration, so it is not used.
    Timed {
        date: (1900, 5, 28),
        station: "Algiers, Maunder",
        at: LatLon {
            lat: 36.80,
            lon: 3.0354,
        },
        second: hms(16.0, 17.0, 17.1),
        third: None,
    },
    // Davidson at Giggleswick, the chronometer set by wireless time signals: "the
    // chronometer showed 5h 25m 0s. The correction to the chronometer was −44s·5, so that
    // this corresponded to 5h 24m 15s·5 G.M.T." (MNRAS 87, 657, 1927).
    Timed {
        date: (1927, 6, 29),
        station: "Giggleswick, Davidson",
        at: LatLon {
            lat: 54.07,
            lon: -2.295,
        },
        second: hms(5.0, 24.0, 15.5),
        third: None,
    },
];

/// How far one timing may lie from the computed contact, seconds: the observers' clocks
/// and reactions, and the Moon's mountains, which the computed smooth limb ignores.
const ONE_TIMING_S: f64 = 8.0;

/// How far the mean of all timings may lie from the computed contacts, seconds. The USNO's
/// Delta T puts it near three.
const MEAN_S: f64 = 2.0;

#[test]
fn contacts_as_the_observers_timed_them() {
    let mut residuals = Vec::new();
    for timed in &TIMED {
        let (year, month, day) = timed.date;
        let midnight = CalendarTime::jd_at_midnight(year, month, day);
        let eclipse = SolarEclipse::near(midnight + 0.5).unwrap().unwrap();
        let local = eclipse.local(timed.at).unwrap().unwrap();
        let ut = |tt: f64| (tt - midnight) * 86_400.0 - eclipse.delta_t.seconds;
        let mut pairs = vec![(timed.second, local.c2.unwrap().tt)];
        if let Some(third) = timed.third {
            pairs.push((third, local.c3.unwrap().tt));
        }
        for (observed, computed) in pairs {
            let residual = observed - ut(computed);
            println!(
                "{year} {}: observed minus computed {residual:+.1} s",
                timed.station
            );
            residuals.push((timed.station, residual));
        }
    }
    let mean = residuals.iter().map(|(_, r)| r).sum::<f64>() / residuals.len() as f64;
    println!("mean {mean:+.1} s over {} timings", residuals.len());
    let far: Vec<_> = residuals
        .iter()
        .filter(|(_, r)| r.abs() > ONE_TIMING_S)
        .collect();
    assert!(far.is_empty(), "far from the record: {far:?}");
    assert!(mean.abs() <= MEAN_S, "timings biased by {mean:+.1} s");
}
