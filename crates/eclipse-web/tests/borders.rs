//! The borders the basemap strokes are those Ukraine's worldview in Natural Earth draws
//! between countries: none across the Perekop isthmus between Crimea and the mainland, while
//! the Kerch Strait, between Crimea and Russia, keeps its line.

const BORDERS: &str = include_str!("../assets/borders.geojson");

/// Every vertex of the borders, as longitude and latitude.
#[cfg(test)]
fn vertices() -> Vec<(f64, f64)> {
    let doc: serde_json::Value = serde_json::from_str(BORDERS).unwrap();
    let mut points = Vec::new();
    for feature in doc["features"].as_array().unwrap() {
        let geometry = &feature["geometry"];
        let lines = match geometry["type"].as_str().unwrap() {
            "LineString" => vec![&geometry["coordinates"]],
            _ => geometry["coordinates"].as_array().unwrap().iter().collect(),
        };
        for line in lines {
            for p in line.as_array().unwrap() {
                points.push((p[0].as_f64().unwrap(), p[1].as_f64().unwrap()));
            }
        }
    }
    points
}

#[test]
fn no_border_divides_crimea_from_the_mainland() {
    let points = vertices();
    let perekop =
        |&&(lon, lat): &&(f64, f64)| (33.4..35.2).contains(&lon) && (45.6..46.4).contains(&lat);
    let kerch =
        |&&(lon, lat): &&(f64, f64)| (36.4..36.8).contains(&lon) && (44.9..45.7).contains(&lat);
    assert_eq!(points.iter().filter(perekop).count(), 0);
    assert!(points.iter().filter(kerch).count() > 5);
    assert!(points.len() > 10_000, "{} vertices", points.len());
}
