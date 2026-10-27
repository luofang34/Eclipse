//! The basemap style: a white globe with pale land, coastlines and borders, from Natural
//! Earth's 1:50m data.

use serde_json::{Value, json};

use crate::basemap::{BORDERS, COAST, LAND};

/// The basemap as a MapLibre style document.
pub(super) fn basemap() -> Result<Value, serde_json::Error> {
    let land: Value = serde_json::from_str(LAND)?;
    let coast: Value = serde_json::from_str(COAST)?;
    let borders: Value = serde_json::from_str(BORDERS)?;
    Ok(json!({
        "version": 8,
        "name": "Eclipse basemap",
        "center": [0.0, 20.0],
        "zoom": 0.5,
        "projection": {"type": "vertical-perspective"},
        "sources": {
            "land": {"type": "geojson", "data": land},
            "coast": {"type": "geojson", "data": coast},
            "borders": {"type": "geojson", "data": borders},
        },
        "layers": [
            {"id": "ocean", "type": "background", "paint": {"background-color": "#ffffff"}},
            {"id": "land", "type": "fill", "source": "land",
             "paint": {"fill-color": "#f4f1ea"}},
            {"id": "coast", "type": "line", "source": "coast",
             "paint": {"line-color": "#202020", "line-width": 0.8}},
            {"id": "borders", "type": "line", "source": "borders",
             "paint": {"line-color": "#e39a2d", "line-width": 0.8}},
        ],
    }))
}
