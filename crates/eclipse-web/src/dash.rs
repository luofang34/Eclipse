//! Dashes that belong to a curve on the ground. Laid out by distance along the whole curve,
//! they stay where they are as the globe turns; measured on screen from the first visible
//! point, as a stroke's dashes are, they would slide along it.

use eclipse_core::geodesy::{LatLon, distance_km, overhead, up};

#[cfg(test)]
mod tests;

/// The dash and gap the eye should see at the middle of the view, screen points.
const DASH_POINTS: f64 = 6.0;
const GAP_POINTS: f64 = 5.0;

/// The dash's share of a period.
const DASH_SHARE: f64 = DASH_POINTS / (DASH_POINTS + GAP_POINTS);

/// The distance in km from one dash to the next at `points_per_km`. A power of two, so zooming
/// leaves the pattern alone until it halves, when every other dash keeps its start.
pub(crate) fn period_km(points_per_km: f64) -> f64 {
    ((DASH_POINTS + GAP_POINTS) / points_per_km)
        .log2()
        .round()
        .exp2()
}

/// The dashes of `curve` every `period` km, as polylines. A segment `drawn` turns down gets
/// none, but still counts toward the distance, so the dashes elsewhere stay put.
pub(crate) fn dashes(
    curve: &[LatLon],
    period: f64,
    mut drawn: impl FnMut(LatLon, LatLon) -> bool,
) -> Vec<Vec<LatLon>> {
    let mut all = Vec::new();
    let mut dash = Vec::new();
    let mut start = 0.0;
    for pair in curve.windows(2) {
        let &[a, b] = pair else { continue };
        let length = distance_km(a, b);
        let end = start + length;
        if length > 0.0 && drawn(a, b) {
            let at = |km: f64| between(a, b, (km - start) / length);
            let mut cycle = (start / period).floor();
            while cycle * period < end {
                let off = (cycle + DASH_SHARE) * period;
                let (from, to) = ((cycle * period).max(start), off.min(end));
                if from < to {
                    if dash.is_empty() {
                        dash.push(at(from));
                    }
                    dash.push(at(to));
                }
                if off <= end && !dash.is_empty() {
                    all.push(std::mem::take(&mut dash));
                }
                cycle += 1.0;
            }
        } else if !dash.is_empty() {
            all.push(std::mem::take(&mut dash));
        }
        start = end;
    }
    if !dash.is_empty() {
        all.push(dash);
    }
    all
}

/// The point the share `t` of the way from `a` to `b`.
fn between(a: LatLon, b: LatLon, t: f64) -> LatLon {
    let (a, b) = (up(a), up(b));
    overhead([
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ])
}
