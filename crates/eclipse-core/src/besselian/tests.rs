use super::*;
use crate::{FLATTENING, bodies, geodesy::norm};

fn on_ellipsoid(p: [f64; 3]) -> f64 {
    let c = 1.0 / (1.0 - FLATTENING);
    (p[0] * p[0] + p[1] * p[1] + p[2] * p[2] * c * c).sqrt() - 1.0
}

#[test]
fn surface_and_outline_points_lie_on_the_ellipsoid() {
    let e = Elements::new(&bodies(2_461_265.24).unwrap());
    let p = e.surface(0.3, -0.4).unwrap();
    assert!(on_ellipsoid(p).abs() < 1e-12);
    let [xi, eta, zeta] = e.project(p);
    assert!((xi - 0.3).abs() < 1e-12 && (eta + 0.4).abs() < 1e-12 && zeta > 0.0);
    assert!(e.surface(1.2, 0.0).is_none());
    for i in 0..36 {
        let q = e.outline(f64::from(i) * 10f64.to_radians());
        assert!(on_ellipsoid(q).abs() < 1e-9, "{}", on_ellipsoid(q));
        // The line along the axis only touches the Earth at the outline.
        assert!(
            e.surface(e.project(q)[0] * 1.000_001, e.project(q)[1] * 1.000_001)
                .is_none()
        );
        assert!(norm(q) > 0.99);
    }
}

#[test]
fn shadow_matches_the_2026_august_total_eclipse() {
    // The Canon gives gamma 0.8977 and magnitude 1.0386 at greatest eclipse, 17:47:06 TD.
    let e = Elements::new(&bodies(2_461_265.240_93).unwrap());
    let gamma = e.x.hypot(e.y);
    assert!((gamma - 0.8977).abs() < 0.001, "gamma {gamma}");
    assert!(e.l2 < 0.0, "a total eclipse has a negative l2");
    let p = e.surface(e.x, e.y).unwrap();
    let shadow = e.shadow(p);
    assert!(shadow.central());
    let ratio = shadow.diameter_ratio();
    assert!((ratio - 1.0386).abs() < 0.0005, "{ratio}");
    assert!((shadow.magnitude() - (1.0 + ratio) / 2.0).abs() < 1e-12);
    assert!((shadow.obscuration() - 1.0).abs() < 1e-12);
}
