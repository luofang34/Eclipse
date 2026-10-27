use super::*;
use eclipse_core::{
    EARTH_RADIUS_M, SolarEclipse,
    geodesy::surface,
    time::{CalendarTime, tt_epoch},
};

/// The renderer's sphere: MapLibre's `Body::EARTH`.
const RADIUS: f64 = 6_371_008.8;

fn angle(a: Vector3<f64>, b: Vector3<f64>) -> f64 {
    a.normalize()
        .dot(b.normalize())
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// The ECEF direction of a vector in the renderer's axes, as Sokoly-App's
/// `globe_world_to_ecef` reads it back.
fn ecef(v: Vector3<f64>) -> [f64; 3] {
    [v.z, v.x, v.y]
}

/// The camera's right, in the renderer's axes.
fn right_of(pose: &Pose) -> Vector3<f64> {
    let [w, x, y, z] = pose.orientation;
    Quaternion::new(w, x, y, z)
        .conjugate()
        .rotate_vector(Vector3::unit_x())
}

#[test]
fn ecef_axes_turn_into_the_renderers() {
    // 0°N 0°E is the renderer's z and the north pole its y, as MapLibre's
    // `ecef_to_globe_world` has them.
    assert_eq!(globe_world([1.0, 0.0, 0.0]), Vector3::unit_z());
    assert_eq!(globe_world([0.0, 0.0, 1.0]), Vector3::unit_y());
    assert_eq!(ecef(globe_world([0.3, -0.5, 0.8])), [0.3, -0.5, 0.8]);
}

#[test]
fn directions_on_the_horizon_and_overhead() {
    let site = LatLon { lat: 0.0, lon: 0.0 };
    let close = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    for read in [on_sphere, horizontal] {
        assert!(close(read(site, [0.0, 1.0, 0.0]), [90.0, 0.0]), "east");
        assert!(close(read(site, [0.0, 0.0, 1.0]), [0.0, 0.0]), "north");
        assert!(
            (read(site, [1.0, 0.0, 0.0])[1] - 90.0).abs() < 1e-9,
            "zenith"
        );
    }
}

#[test]
fn a_standing_eye_looks_where_it_is_told_with_the_horizon_level() {
    let views = [
        // The MapLibre fork's look-up test, Sokoly's moonlight case: the Moon at azimuth
        // 92.175°, elevation 37.001° from 39.04855° N, 77.11311° W.
        (39.04855, -77.11311, 92.175, 37.001),
        (65.2, -25.2, 248.0, 25.7),
        (-33.9, 18.4, 10.0, 0.0),
        (6.4, -170.7, 7.0, 89.5),
    ];
    for (lat, lon, azimuth, elevation) in views {
        let site = LatLon { lat, lon };
        let pose = standing(site, azimuth, elevation, RADIUS);
        let (eye, forward) = eye_and_forward(&pose, RADIUS);
        assert!((eye.magnitude() - RADIUS - EYE_HEIGHT_M).abs() < 1e-3);
        let [a, e] = on_sphere(site, ecef(forward));
        assert!((e - elevation).abs() < 1e-6, "{lat} {lon}: elevation {e}");
        assert!(
            ((a - azimuth + 540.0) % 360.0 - 180.0).abs() < 1e-5,
            "azimuth {a}"
        );
        // No roll: the camera's right lies in the horizon.
        assert!(right_of(&pose).dot(eye.normalize()).abs() < 1e-9);
        assert!(pose.target_elevation_meters > 0.0);
    }
}

#[test]
fn the_view_turns_to_navigates_sun_at_greatest_eclipse() {
    let t = CalendarTime::jd_at_midnight(2026, 8, 12) + 0.74;
    let e = SolarEclipse::near(t).unwrap().unwrap();
    let site = e.greatest;
    let eye = surface(site).map(|c| c * EARTH_RADIUS_M);
    let sun = navigate_celestial::sun_ecef_tt(tt_epoch(e.greatest_tt), Some(eye))
        .unwrap()
        .direction;
    // Against the true horizon the Sun stands where eclipse-core's own geometry puts it.
    assert!((horizontal(site, sun)[1] - e.sun_altitude).abs() < 0.05);
    // Aimed by the sphere's horizon, the renderer's camera looks straight at it.
    let [azimuth, elevation] = on_sphere(site, sun);
    let (_, forward) = eye_and_forward(&standing(site, azimuth, elevation, RADIUS), RADIUS);
    assert!(angle(forward, globe_world(sun)) < 1e-6);
}

#[test]
fn the_flight_leaves_its_first_pose_and_lands_on_its_last_above_the_ground() {
    // Over the globe: 20 000 km above 0°N 0°E, looking at the Earth's centre.
    let high = globe_world([1.0, 0.0, 0.0]) * (RADIUS + 2.0e7);
    let from = pose_at(high, -high, Vector3::unit_y(), 2.0e7, RADIUS);
    let to = standing(
        LatLon {
            lat: 40.4,
            lon: -3.7,
        },
        101.0,
        39.8,
        RADIUS,
    );
    for i in 0..=40 {
        let pose = between(&from, &to, f64::from(i) / 40.0, RADIUS);
        assert!(pose.target_elevation_meters > 0.0, "step {i}");
    }
    for (t, end) in [(0.0, &from), (1.0, &to)] {
        let (eye, forward) = eye_and_forward(&between(&from, &to, t, RADIUS), RADIUS);
        let (eye_end, forward_end) = eye_and_forward(end, RADIUS);
        assert!((eye - eye_end).magnitude() < 0.01, "eye at {t}");
        assert!(angle(forward, forward_end) < 1e-6, "forward at {t}");
    }
}

/// TT for a UTC Julian date in 2026: TAI - UTC is 37 s and TT - TAI 32.184 s.
fn tt_from_utc(jd_utc: f64) -> f64 {
    jd_utc + 69.184 / 86_400.0
}

/// Sokoly-Demos' sky cases, which this view's fly-in follows: the eye lands at the observer
/// looking along the body's azimuth and elevation. Navigate's Sun and Moon, as this view
/// takes them, stand there, and the standing pose looks straight at them.
#[test]
fn sokoly_demos_sunshine_and_moonlight() {
    let sunshine = (
        "Sunshine",
        LatLon {
            lat: 41.07204,
            lon: -71.85733,
        },
        CalendarTime::jd_at_midnight(2026, 6, 14) + (9.0 + 19.0 / 60.0 + 39.647 / 3600.0) / 24.0,
        [58.397, 0.0],
        true,
    );
    let moonlight = (
        "Moonlight",
        LatLon {
            lat: 39.04855,
            lon: -77.11311,
        },
        CalendarTime::jd_at_midnight(2026, 10, 27) + (1.0 + 42.0 / 60.0) / 24.0,
        [92.175, 37.001],
        false,
    );
    for (name, site, jd_utc, [azimuth, elevation], sun) in [sunshine, moonlight] {
        let epoch = tt_epoch(tt_from_utc(jd_utc));
        let eye = surface(site).map(|c| c * EARTH_RADIUS_M);
        let sight = if sun {
            navigate_celestial::sun_ecef_tt(epoch, Some(eye))
        } else {
            navigate_celestial::moon_ecef_tt(epoch, Some(eye))
        };
        let direction = sight.unwrap().direction;
        let [a, e] = horizontal(site, direction);
        println!("{name}: azimuth {a:.3}°, elevation {e:.3}°");
        assert!((a - azimuth).abs() < 0.01, "{name} azimuth {a}");
        assert!((e - elevation).abs() < 0.01, "{name} elevation {e}");
        let [sphere_azimuth, sphere_elevation] = on_sphere(site, direction);
        let pose = standing(site, sphere_azimuth, sphere_elevation, RADIUS);
        let (_, forward) = eye_and_forward(&pose, RADIUS);
        assert!(angle(forward, globe_world(direction)) < 1e-6, "{name}");
    }
}
