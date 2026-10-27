//! Curves traced through time: at each instant a family gives the points of its curve that
//! exist then, and points of consecutive instants are joined into polylines.

use std::f64::consts::TAU;

use crate::geodesy::{norm, sub};

/// The finest time step refinement reaches where a curve begins or ends, days.
const FINEST_STEP: f64 = 0.25 / 86_400.0;
/// Points further apart than this in consecutive instants are not one curve, Earth radii.
const LINK_DISTANCE: f64 = 0.15;
/// Instants are added until consecutive points are closer than this, so a polyline follows
/// its curve where the curve runs fast, Earth radii (about 50 km).
const SMOOTH_DISTANCE: f64 = 0.008;
/// Curve ends closer than this are one curve, Earth radii (about 20 km).
const JOIN_DISTANCE: f64 = 0.003;

/// The roots on `[0, 2π)` of a function sampled at `n` angles, where it is defined at both
/// ends of a sign change.
pub(super) fn circular_roots(n: usize, f: impl Fn(f64) -> Option<f64>) -> Vec<f64> {
    let step = TAU / n as f64;
    let values: Vec<Option<f64>> = (0..n).map(|i| f(i as f64 * step)).collect();
    let mut roots = Vec::new();
    for i in 0..n {
        let (Some(Some(a)), Some(Some(b))) = (values.get(i), values.get((i + 1) % n)) else {
            continue;
        };
        if a.signum() == b.signum() {
            continue;
        }
        let (mut lo, mut hi, mut f_lo) = (i as f64 * step, (i + 1) as f64 * step, *a);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            match f(mid) {
                Some(v) if v.signum() == f_lo.signum() => {
                    lo = mid;
                    f_lo = v;
                }
                Some(_) => hi = mid,
                None => break,
            }
        }
        roots.push(0.5 * (lo + hi));
    }
    roots
}

/// The points of a curve at each instant of `[start, end]`, `step` apart, with instants
/// added where the number of points changes, so curves run to where they begin and end.
pub(super) fn sweep<E>(
    start: f64,
    end: f64,
    step: f64,
    at: impl Fn(f64) -> Result<Vec<[f64; 3]>, E>,
) -> Result<Vec<Vec<[f64; 3]>>, E> {
    let count = ((end - start) / step).ceil().max(1.0) as usize;
    let mut out = Vec::new();
    let mut previous = (start, at(start)?);
    for i in 1..=count {
        let t = (start + i as f64 * step).min(end);
        let next = (t, at(t)?);
        out.push(previous.1.clone());
        refine(&previous, &next, &at, &mut out)?;
        previous = next;
    }
    out.push(previous.1);
    Ok(out)
}

/// Adds the instants between `a` and `b` that bracket a change in the number of points.
fn refine<E>(
    a: &(f64, Vec<[f64; 3]>),
    b: &(f64, Vec<[f64; 3]>),
    at: &impl Fn(f64) -> Result<Vec<[f64; 3]>, E>,
    out: &mut Vec<Vec<[f64; 3]>>,
) -> Result<(), E> {
    if b.0 - a.0 <= FINEST_STEP || !jumps(&a.1, &b.1) {
        return Ok(());
    }
    let t = 0.5 * (a.0 + b.0);
    let mid = (t, at(t)?);
    refine(a, &mid, at, out)?;
    out.push(mid.1.clone());
    refine(&mid, b, at, out)
}

/// Whether a curve begins or ends between two instants, or moves too far to be followed.
fn jumps(a: &[[f64; 3]], b: &[[f64; 3]]) -> bool {
    a.len() != b.len()
        || b.iter().any(|p| {
            a.iter()
                .map(|q| norm(sub(*p, *q)))
                .fold(f64::INFINITY, f64::min)
                > SMOOTH_DISTANCE
        })
}

/// Joins the points of consecutive instants into polylines, each point to the nearest end
/// of a curve still running, then joins curves whose ends meet.
pub(super) fn link(instants: Vec<Vec<[f64; 3]>>) -> Vec<Vec<[f64; 3]>> {
    let mut finished: Vec<Vec<[f64; 3]>> = Vec::new();
    let mut running: Vec<Vec<[f64; 3]>> = Vec::new();
    for points in instants {
        let mut next: Vec<Vec<[f64; 3]>> = Vec::new();
        let mut unclaimed: Vec<Option<[f64; 3]>> = points.into_iter().map(Some).collect();
        // Nearest pairs first, so crossing curves keep their own points.
        let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
        for (c, curve) in running.iter().enumerate() {
            let Some(end) = curve.last() else { continue };
            for (p, point) in unclaimed.iter().enumerate() {
                if let Some(point) = point {
                    pairs.push((norm(sub(*end, *point)), c, p));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut taken = vec![false; running.len()];
        let mut extended: Vec<Option<Vec<[f64; 3]>>> = running.into_iter().map(Some).collect();
        for (distance, c, p) in pairs {
            if distance > LINK_DISTANCE || taken.get(c).copied().unwrap_or(true) {
                continue;
            }
            let Some(point) = unclaimed.get_mut(p).and_then(Option::take) else {
                continue;
            };
            if let (Some(flag), Some(Some(curve))) = (taken.get_mut(c), extended.get_mut(c)) {
                *flag = true;
                curve.push(point);
            }
        }
        for (curve, taken) in extended.into_iter().zip(taken) {
            match (curve, taken) {
                (Some(curve), true) => next.push(curve),
                (Some(curve), false) => finished.push(curve),
                (None, _) => {}
            }
        }
        next.extend(unclaimed.into_iter().flatten().map(|point| vec![point]));
        running = next;
    }
    finished.extend(running);
    join(finished)
}

/// Joins curves whose ends meet, as a loop's two halves meet where it is born and dies.
fn join(mut curves: Vec<Vec<[f64; 3]>>) -> Vec<Vec<[f64; 3]>> {
    let mut out: Vec<Vec<[f64; 3]>> = Vec::new();
    while let Some(mut curve) = curves.pop() {
        loop {
            let ends = |c: &Vec<[f64; 3]>| (c.first().copied(), c.last().copied());
            let found = curves.iter().position(|other| {
                let (Some(a), Some(b)) = ends(&curve) else {
                    return false;
                };
                let (Some(c), Some(d)) = ends(other) else {
                    return false;
                };
                [(a, c), (a, d), (b, c), (b, d)]
                    .iter()
                    .any(|(p, q)| norm(sub(*p, *q)) < JOIN_DISTANCE)
            });
            let Some(index) = found else { break };
            let mut other = curves.swap_remove(index);
            let (Some(a), Some(b), Some(c), Some(d)) =
                (curve.first(), curve.last(), other.first(), other.last())
            else {
                break;
            };
            let close = |p: &[f64; 3], q: &[f64; 3]| norm(sub(*p, *q)) < JOIN_DISTANCE;
            if close(b, c) {
                curve.extend(other);
            } else if close(b, d) {
                other.reverse();
                curve.extend(other);
            } else if close(a, d) {
                other.extend(curve);
                curve = other;
            } else {
                other.reverse();
                other.extend(curve);
                curve = other;
            }
        }
        if curve.len() > 1 {
            out.push(curve);
        }
    }
    out
}
