use eclipse_core::geodesy::{LatLon, distance_km};

use super::{DASH_POINTS, GAP_POINTS, dashes, period_km};

/// A curve along the equator, a point every degree of longitude, crossing the antimeridian.
fn equator() -> Vec<LatLon> {
    (0..=120)
        .map(|i| LatLon {
            lat: 0.0,
            lon: (120.0 + f64::from(i) + 180.0).rem_euclid(360.0) - 180.0,
        })
        .collect()
}

const PERIOD_KM: f64 = 128.0;

fn same_point(a: LatLon, b: LatLon) -> bool {
    distance_km(a, b) < 1e-6
}

#[test]
fn dashes_keep_their_ground_positions_as_the_view_cuts_the_curve() {
    let curve = equator();
    let whole = dashes(&curve, PERIOD_KM, |_, _| true);
    // As the globe turns, the stretch on screen starts further along the curve.
    for hidden_to in [10.0, 37.5, 81.0] {
        let visible = dashes(&curve, PERIOD_KM, |a, _| {
            (a.lon + 240.0).rem_euclid(360.0) >= hidden_to
        });
        assert!(!visible.is_empty());
        for dash in visible.iter().skip(1) {
            let first = *dash.first().unwrap();
            let last = *dash.last().unwrap();
            assert!(
                whole.iter().any(|w| {
                    same_point(*w.first().unwrap(), first) && same_point(*w.last().unwrap(), last)
                }),
                "a dash from {first:?} to {last:?} moved with the view from {hidden_to}°",
            );
        }
    }
}

#[test]
fn a_dash_spans_segments_and_the_pattern_keeps_its_share() {
    let whole = dashes(&equator(), PERIOD_KM, |_, _| true);
    let share = DASH_POINTS / (DASH_POINTS + GAP_POINTS);
    for pair in whole.windows(2) {
        let [dash, next] = pair else { continue };
        let (start, end) = (*dash.first().unwrap(), *dash.last().unwrap());
        assert!((distance_km(start, end) - share * PERIOD_KM).abs() < 0.2);
        let to_next = distance_km(start, *next.first().unwrap());
        assert!((to_next - PERIOD_KM).abs() < 0.2, "period {to_next}");
    }
    assert!(whole.iter().any(|dash| dash.len() > 2));
}

#[test]
fn the_dash_period_turns_with_the_zoom_and_not_within_a_step() {
    // 0.1 points per km is a globe of about 640 points' radius.
    let period = period_km(0.1);
    assert_eq!(period, 128.0);
    assert_eq!(period_km(0.095), period);
    assert_eq!(period_km(0.105), period);
    assert_eq!(period_km(0.2), period / 2.0);
    assert_eq!(period_km(0.4), period / 4.0);
}

#[test]
fn a_short_curve_gets_a_dash_and_an_empty_one_none() {
    let short = [LatLon { lat: 0.0, lon: 0.0 }, LatLon { lat: 0.0, lon: 0.1 }];
    assert_eq!(dashes(&short, PERIOD_KM, |_, _| true).len(), 1);
    assert!(dashes(&[], PERIOD_KM, |_, _| true).is_empty());
}
