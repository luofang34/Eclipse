use super::*;

#[test]
fn interpolation_matches_navigate_to_metres() {
    let start = 2_461_265.0;
    let track = Track::sample(start, start + 0.4).unwrap();
    for i in 0..37 {
        let jd = start + 0.0107 * f64::from(i);
        let direct = bodies(jd).unwrap();
        let interpolated = track.at(jd).unwrap();
        for axis in 0..3 {
            let moon = (direct.moon[axis] - interpolated.moon[axis]) * EARTH_RADIUS_M;
            let sun = (direct.sun[axis] - interpolated.sun[axis]) * EARTH_RADIUS_M;
            assert!(moon.abs() < 20.0, "Moon off by {moon} m at {jd}");
            assert!(sun.abs() < 2_000.0, "Sun off by {sun} m at {jd}");
        }
    }
}

#[test]
fn outside_the_window_is_an_error() {
    let track = Track::sample(2_461_265.0, 2_461_265.1).unwrap();
    assert!(matches!(
        track.at(2_461_266.0),
        Err(EclipseError::OutsideWindow { .. })
    ));
}
