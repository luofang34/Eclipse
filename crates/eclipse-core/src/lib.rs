//! Solar and lunar eclipses from Navigate's analytic Sun and Moon: when they happen, their
//! circumstances, and the curves of NASA-style global maps.

pub mod besselian;
mod ephemeris;
mod error;
pub mod geodesy;
mod lunar;
pub mod saros;
mod search;
mod solar;
pub mod time;

pub use ephemeris::{Bodies, EARTH_RADIUS_M, FLATTENING, Track, bodies};
pub use error::EclipseError;
pub use lunar::{LunarContacts, LunarEclipse, LunarKind, MoonInShadow};
pub use search::{Eclipse, eclipses_in_year};
pub use solar::{Contact, LocalSolar, SolarEclipse, SolarKind, SolarMap};
