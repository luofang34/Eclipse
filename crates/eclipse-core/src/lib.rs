//! Solar and lunar eclipses from Navigate's analytic Sun and Moon: when they happen, their
//! circumstances, and the curves of NASA-style global maps.

pub mod besselian;
mod ephemeris;
mod error;
pub mod geodesy;
pub mod saros;
mod solar;
pub mod time;

pub use ephemeris::{Bodies, EARTH_RADIUS_M, FLATTENING, Track, bodies};
pub use error::EclipseError;
pub use solar::{Contact, LocalSolar, SolarEclipse, SolarKind, SolarMap};
