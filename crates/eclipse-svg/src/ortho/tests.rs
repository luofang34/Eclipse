use super::*;

const VIEW: LatLon = LatLon {
    lat: 40.0,
    lon: -100.0,
};

fn view() -> Ortho {
    Ortho::new(VIEW, 100.0, 100.0, 50.0)
}

/// The area a closed path of absolute `M`/`L` commands encloses.
fn area(d: &str) -> f64 {
    let points: Vec<[f64; 2]> = d
        .trim_end_matches('Z')
        .split(['M', 'L'])
        .filter(|s| !s.is_empty())
        .map(|pair| {
            let mut it = pair.split(' ').map(|v| v.parse::<f64>().unwrap());
            [it.next().unwrap(), it.next().unwrap()]
        })
        .collect();
    let mut twice = 0.0;
    for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
        twice += a[0] * b[1] - b[0] * a[1];
    }
    (twice / 2.0).abs()
}

#[test]
fn the_centre_projects_to_the_middle_and_north_is_up() {
    let [x, y] = view().project(VIEW).unwrap();
    assert!((x - 100.0).abs() < 1e-9 && (y - 100.0).abs() < 1e-9);
    let [_, y] = view()
        .project(LatLon {
            lat: 50.0,
            lon: -100.0,
        })
        .unwrap();
    assert!(y < 100.0);
    let [x, _] = view()
        .project(LatLon {
            lat: 40.0,
            lon: -90.0,
        })
        .unwrap();
    assert!(x > 100.0);
}

#[test]
fn the_far_side_is_hidden_and_the_limb_lies_on_the_radius() {
    let anti = LatLon {
        lat: -40.0,
        lon: 80.0,
    };
    assert!(view().project(anti).is_none());
    let limb = LatLon {
        lat: -50.0,
        lon: -100.0,
    };
    let [x, y] = view().project(limb).unwrap();
    assert!(((x - 100.0).hypot(y - 100.0) - 50.0).abs() < 1e-6);
}

#[test]
fn a_curve_breaks_where_it_passes_behind_the_globe() {
    let equator: Vec<LatLon> = (-180..=180)
        .map(|lon| LatLon {
            lat: 0.0,
            lon: f64::from(lon),
        })
        .collect();
    let d = Ortho::new(LatLon { lat: 0.0, lon: 0.0 }, 0.0, 0.0, 1.0).polyline_d(&[equator], 0.0);
    // Visible from 90° W to 90° E only: one stretch.
    assert_eq!(d.matches('M').count(), 1);
}

#[test]
fn a_hemisphere_covers_half_the_disc_when_its_pole_is_on_the_limb() {
    let full = std::f64::consts::PI * 50.0 * 50.0;
    let pole = LatLon {
        lat: -50.0,
        lon: -100.0,
    };
    let half = area(&view().hemisphere_d(pole).unwrap());
    assert!((half / full - 0.5).abs() < 0.01, "share {}", half / full);
    let whole = area(&view().hemisphere_d(VIEW).unwrap());
    assert!((whole / full - 1.0).abs() < 0.01);
    assert!(
        view()
            .hemisphere_d(LatLon {
                lat: -40.0,
                lon: 80.0
            })
            .is_none()
    );
}
