//! The coast the basemap strokes is shore only: none of the edges that close Natural Earth's
//! land polygons along the antimeridian or across the south pole, which the globe would draw
//! as lines where there is no coast.

use natural_earth_50m::COAST;

#[test]
fn the_coast_has_no_cut_edges() {
    let doc: serde_json::Value = serde_json::from_str(COAST).unwrap();
    let lines = doc["features"][0]["geometry"]["coordinates"]
        .as_array()
        .unwrap();
    let mut segments = 0;
    for line in lines {
        let points: Vec<(f64, f64)> = line
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap()))
            .collect();
        for pair in points.windows(2) {
            let ((lon_a, lat_a), (lon_b, lat_b)) = (pair[0], pair[1]);
            assert!(!(lon_a.abs() == 180.0 && lon_b.abs() == 180.0), "{pair:?}");
            assert!(!(lat_a < -85.0 && lat_b < -85.0), "{pair:?}");
            segments += 1;
        }
    }
    assert!(segments > 50_000, "{segments} segments");
}
