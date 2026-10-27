//! Totality as observers timed it at historic eclipses, against the duration computed at
//! their stations. The Canon tests compare with NASA's computed eclipses; these compare
//! with what was seen.

use eclipse_core::{SolarEclipse, geodesy::LatLon, time::CalendarTime};

/// A timed observation: the eclipse's date, the station and the totality recorded there.
struct Observed {
    date: (i32, u8, u8),
    station: &'static str,
    at: LatLon,
    totality_s: f64,
    /// How far the record can be trusted: its rounding, and how it was timed.
    tolerance_s: f64,
}

const OBSERVED: [Observed; 3] = [
    // Berkowski's daguerreotype, the first photograph of totality: "during the 2 minutes
    // 56 seconds the Moon obscured the Sun at the Royal Observatory in Königsberg"
    // (Astronomy, "July 28, 1851: The first photo of a total eclipse").
    Observed {
        date: (1851, 7, 28),
        station: "Königsberg",
        at: LatLon {
            lat: 54.714,
            lon: 20.500,
        },
        totality_s: 176.0,
        tolerance_s: 3.0,
    },
    // Crommelin and Davidson's station for the light deflection test: "during the 5 minute
    // and 13 seconds eclipse" (Lemos, "Shadow of the Moon and general relativity",
    // arXiv:1912.05587, after Dyson, Eddington and Davidson, Phil. Trans. A 220, 1920).
    Observed {
        date: (1919, 5, 29),
        station: "Sobral",
        at: LatLon {
            lat: -3.686,
            lon: -40.349,
        },
        totality_s: 313.0,
        tolerance_s: 3.0,
    },
    // Eddington and Cottingham at Roça Sundy: "the beat of the metronome ticking out the
    // 302 seconds of totality" (Eddington, Space, Time and Gravitation, 1920). A count by
    // metronome allows a few beats either way.
    Observed {
        date: (1919, 5, 29),
        station: "Príncipe",
        at: LatLon {
            lat: 1.672,
            lon: 7.388,
        },
        totality_s: 302.0,
        tolerance_s: 5.0,
    },
];

#[test]
fn totality_as_observers_timed_it() {
    let mut failures = Vec::new();
    for observed in &OBSERVED {
        let (year, month, day) = observed.date;
        let noon = CalendarTime::jd_at_midnight(year, month, day) + 0.5;
        let eclipse = SolarEclipse::near(noon).unwrap().unwrap();
        let local = eclipse.local(observed.at).unwrap().unwrap();
        let computed = local.central_duration_s.unwrap();
        println!(
            "{year}-{month:02}-{day:02} {}: computed {computed:.1} s, observed {} s",
            observed.station, observed.totality_s
        );
        if (computed - observed.totality_s).abs() > observed.tolerance_s {
            failures.push(observed.station);
        }
    }
    assert!(failures.is_empty(), "differ from the record: {failures:?}");
}
