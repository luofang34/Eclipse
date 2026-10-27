use super::*;
use crate::time::CalendarTime;

/// Canon rows: date, TD of greatest eclipse, type, gamma, magnitude, latitude, longitude,
/// Sun altitude, path width (km), central duration (s), Saros.
type Row = (
    &'static str,
    &'static str,
    char,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    i64,
);

const CANON: &[Row] = &[
    (
        "1851-07-28",
        "14:33:42",
        'T',
        0.7644,
        1.0577,
        68.0,
        -20.0,
        40.0,
        296.0,
        221.0,
        143,
    ),
    (
        "1860-07-18",
        "14:26:24",
        'T',
        0.5487,
        1.0500,
        52.0,
        -20.0,
        56.0,
        198.0,
        219.0,
        124,
    ),
    (
        "1868-08-18",
        "05:12:10",
        'T',
        -0.0443,
        1.0756,
        11.0,
        102.0,
        88.0,
        245.0,
        407.0,
        133,
    ),
    (
        "1900-05-28",
        "14:53:56",
        'T',
        0.3943,
        1.0249,
        45.0,
        -46.0,
        67.0,
        92.0,
        130.0,
        126,
    ),
    (
        "1919-05-29",
        "13:08:55",
        'T',
        -0.2955,
        1.0719,
        4.0,
        -17.0,
        73.0,
        244.0,
        411.0,
        136,
    ),
    (
        "1944-01-25",
        "15:26:42",
        'T',
        0.2025,
        1.0428,
        -8.0,
        -50.0,
        78.0,
        146.0,
        249.0,
        130,
    ),
    (
        "1999-08-11",
        "11:04:09",
        'T',
        0.5062,
        1.0286,
        45.0,
        24.0,
        59.0,
        112.0,
        143.0,
        145,
    ),
    (
        "2017-08-21",
        "18:26:40",
        'T',
        0.4367,
        1.0306,
        37.0,
        -88.0,
        64.0,
        115.0,
        160.0,
        145,
    ),
    (
        "2024-04-08",
        "18:18:29",
        'T',
        0.3431,
        1.0566,
        25.0,
        -104.0,
        70.0,
        198.0,
        268.0,
        139,
    ),
    (
        "2026-08-12",
        "17:47:06",
        'T',
        0.8977,
        1.0386,
        65.0,
        -25.0,
        26.0,
        294.0,
        138.0,
        126,
    ),
    (
        "2030-06-01",
        "06:29:13",
        'A',
        0.5626,
        0.9443,
        57.0,
        80.0,
        55.0,
        250.0,
        321.0,
        128,
    ),
    (
        "2061-04-20",
        "02:56:49",
        'T',
        0.9578,
        1.0475,
        65.0,
        59.0,
        16.0,
        559.0,
        157.0,
        149,
    ),
];

fn jd(date: &str, time: &str) -> f64 {
    let d: Vec<i32> = date.split('-').map(|x| x.parse().unwrap()).collect();
    let t: Vec<f64> = time.split(':').map(|x| x.parse().unwrap()).collect();
    CalendarTime::jd_at_midnight(d[0], d[1] as u8, d[2] as u8)
        + (t[0] + t[1] / 60.0 + t[2] / 3600.0) / 24.0
}

#[test]
fn greatest_eclipse_matches_the_canon() {
    for &(date, td, kind, gamma, magnitude, lat, lon, alt, width, duration, saros) in CANON {
        let e = SolarEclipse::near(jd(date, td) + 0.4).unwrap().expect(date);
        assert_eq!(e.kind.code(), kind, "{date}");
        assert!((e.gamma - gamma).abs() < 0.001, "{date} gamma {}", e.gamma);
        assert!(
            (e.magnitude - magnitude).abs() < 0.001,
            "{date} magnitude {}",
            e.magnitude
        );
        // The Canon rounds positions and altitudes to whole degrees.
        assert!(
            (e.greatest.lat - lat).abs() <= 0.6,
            "{date} lat {}",
            e.greatest.lat
        );
        assert!(
            (e.greatest.lon - lon).abs() <= 0.6,
            "{date} lon {}",
            e.greatest.lon
        );
        assert!(
            (e.sun_altitude - alt).abs() <= 0.6,
            "{date} altitude {}",
            e.sun_altitude
        );
        let w = e.path_width_km.unwrap();
        assert!((w - width).abs() / width < 0.03, "{date} width {w}");
        let s = e.central_duration_s.unwrap();
        assert!((s - duration).abs() <= 3.0, "{date} duration {s}");
        assert_eq!(e.saros, saros, "{date}");
        // The greatest eclipse is a flat minimum; Navigate's Moon moves it by seconds.
        assert!(
            (e.greatest_tt - jd(date, td)).abs() * 86_400.0 < 60.0,
            "{date} time"
        );
    }
}

#[test]
fn no_eclipse_at_an_ordinary_new_moon() {
    // 2026 May 16: the Moon passes far from the Sun's direction.
    assert!(
        SolarEclipse::near(jd("2026-05-16", "20:00:00"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn local_circumstances_on_the_central_line() {
    let e = SolarEclipse::near(jd("2017-08-21", "18:26:40"))
        .unwrap()
        .unwrap();
    let local = e.local(e.greatest).unwrap().unwrap();
    assert!(local.magnitude > 1.0 && (local.obscuration - 1.0).abs() < 1e-9);
    let (c1, c4) = (local.c1.unwrap(), local.c4.unwrap());
    assert!(c1.tt < local.max.tt && local.max.tt < c4.tt);
    assert!((local.max.tt - e.greatest_tt).abs() * 86_400.0 < 60.0);
    // Far outside the penumbra nothing is seen.
    assert!(
        e.local(LatLon {
            lat: -60.0,
            lon: 120.0
        })
        .unwrap()
        .is_none()
    );
}

#[test]
fn map_of_2026_august() {
    let e = SolarEclipse::near(jd("2026-08-12", "17:47:06"))
        .unwrap()
        .unwrap();
    let map = e.map().unwrap();
    assert_eq!(map.central_line.len(), 1, "one central line");
    assert_eq!(map.umbral_limits.len(), 2, "two umbral limits");
    assert!(!map.penumbral_limits.is_empty());
    assert!(!map.rise_set.is_empty() && !map.max_rise_set.is_empty());
    assert!(!map.half_magnitude.is_empty());
    // The Sun is overhead near 15° N, 86° W at 17:47 TD.
    assert!(
        (map.subsolar.lat - 14.8).abs() < 0.5 && (map.subsolar.lon + 86.0).abs() < 1.5,
        "{:?}",
        map.subsolar
    );
    // The central line runs from near the North Pole through Greenland, Iceland and Spain.
    let line = &map.central_line[0];
    assert!(
        line.iter()
            .any(|p| (p.lat - 64.5).abs() < 1.0 && (p.lon + 22.0).abs() < 3.0),
        "Iceland"
    );
    assert!(
        line.iter()
            .any(|p| (p.lat - 41.0).abs() < 1.5 && (p.lon + 4.0).abs() < 3.0),
        "Spain"
    );
}
