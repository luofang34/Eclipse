//! The untimed total solar eclipses that bound Delta T in Stephenson, Morrison and Hohenkerk,
//! "Measurement of the Earth's rotation: 720 BC to AD 2015", Proc. R. Soc. A 472 (2016),
//! electronic supplement, Table S10: for each place where the Sun was recorded as totally
//! eclipsed, the range of Delta T for which it lay in the path.
//!
//! Navigate's Delta T is that paper's spline, reduced with a Moon close to DE431's. With
//! that Moon, each place must lie in the computed path over the same range of Delta T, to
//! within the step of the search and the paper's rounding: a Moon that runs ahead of or
//! behind the one the records were reduced with moves both ends of every range together.
#![cfg(feature = "long-range")]

use eclipse_core::{SolarEclipse, geodesy::LatLon, time::CalendarTime};

/// The Earth turns 15.041 arcseconds in a second.
const DEGREES_PER_SECOND: f64 = 15.041 / 3600.0;

/// How far the ends of a computed range may lie from the paper's, seconds.
const TOLERANCE_S: f64 = 40.0;

/// A place recorded in totality: its eclipse's date in the Julian calendar, and the paper's
/// bounds on Delta T, seconds.
struct Anchor {
    name: &'static str,
    date: (i32, u8, u8),
    at: LatLon,
    delta_t: (f64, f64),
}

const ANCHORS: [Anchor; 9] = [
    Anchor {
        name: "-135 Babylon",
        date: (-135, 4, 15),
        at: LatLon {
            lat: 32.54,
            lon: 44.42,
        },
        delta_t: (11_220.0, 12_140.0),
    },
    Anchor {
        name: "454 Jiankang",
        date: (454, 8, 10),
        at: LatLon {
            lat: 32.03,
            lon: 118.78,
        },
        delta_t: (6_120.0, 7_900.0),
    },
    Anchor {
        name: "761 Chang'an",
        date: (761, 8, 5),
        at: LatLon {
            lat: 34.27,
            lon: 108.90,
        },
        delta_t: (1_700.0, 3_260.0),
    },
    Anchor {
        name: "1124 Novgorod",
        date: (1124, 8, 11),
        at: LatLon {
            lat: 58.50,
            lon: 31.33,
        },
        delta_t: (980.0, 2_700.0),
    },
    Anchor {
        name: "1133 Reichersberg",
        date: (1133, 8, 2),
        at: LatLon {
            lat: 48.34,
            lon: 13.37,
        },
        delta_t: (520.0, 1_740.0),
    },
    Anchor {
        name: "1133 Augsburg",
        date: (1133, 8, 2),
        at: LatLon {
            lat: 48.37,
            lon: 10.90,
        },
        delta_t: (-60.0, 1_160.0),
    },
    Anchor {
        name: "1241 Reichersberg",
        date: (1241, 10, 6),
        at: LatLon {
            lat: 48.34,
            lon: 13.37,
        },
        delta_t: (460.0, 1_380.0),
    },
    Anchor {
        name: "1241 Stade",
        date: (1241, 10, 6),
        at: LatLon {
            lat: 53.60,
            lon: 9.48,
        },
        delta_t: (620.0, 1_630.0),
    },
    Anchor {
        name: "1567 Rome",
        date: (1567, 4, 9),
        at: LatLon {
            lat: 41.90,
            lon: 12.48,
        },
        delta_t: (145.0, 165.0),
    },
];

#[test]
fn each_place_is_in_the_path_over_the_papers_range() {
    let mut failures = Vec::new();
    for Anchor {
        name,
        date: (year, month, day),
        at,
        delta_t: (lower, upper),
    } in ANCHORS
    {
        let e = SolarEclipse::near(CalendarTime::jd_at_midnight(year, month, day) + 0.5)
            .unwrap()
            .unwrap();
        let (first, last) = central_range(&e, at, lower, upper);
        println!("{name}: central for Delta T {first:.0}..{last:.0} s, paper {lower}..{upper}");
        if (first - lower).abs() > TOLERANCE_S || (last - upper).abs() > TOLERANCE_S {
            failures.push(format!(
                "{name}: {first:.0}..{last:.0} s vs {lower}..{upper}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The least and greatest Delta T, searched in five-second steps from well below `lower` to
/// well above `upper`, for which `at` sees the eclipse as total or annular.
#[cfg(test)]
fn central_range(e: &SolarEclipse, at: LatLon, lower: f64, upper: f64) -> (f64, f64) {
    let (start, step) = (lower - 300.0, 5.0);
    let steps = ((upper - lower + 600.0) / step) as u32;
    let central: Vec<f64> = (0..=steps)
        .map(|i| start + step * f64::from(i))
        .filter(|delta_t| {
            // A Delta T larger than the model's puts the shadow further east, as if the place
            // lay that much further west.
            let shifted = LatLon {
                lat: at.lat,
                lon: at.lon - (delta_t - e.delta_t.seconds) * DEGREES_PER_SECOND,
            };
            e.local(shifted)
                .unwrap()
                .is_some_and(|seen| seen.central_duration_s.is_some())
        })
        .collect();
    (
        central.first().copied().unwrap_or(f64::NAN),
        central.last().copied().unwrap_or(f64::NAN),
    )
}
