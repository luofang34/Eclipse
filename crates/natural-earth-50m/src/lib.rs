//! Natural Earth's 1:50m land, coasts and country borders as GeoJSON strings, small enough to
//! compile into a program and fine enough for a globe: coordinates are rounded to 0.02
//! degrees, about 2 km, and properties are dropped.
//!
//! The coast is the land's shore alone, without the edges that close its polygons along the
//! antimeridian and across the south pole. The borders are those Ukraine's worldview in
//! Natural Earth draws between countries. `natural_earth.py` writes all three from Natural
//! Earth's own files.

mod layers;

pub use layers::{BORDERS, COAST, LAND};
