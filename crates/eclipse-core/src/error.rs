//! Why an eclipse could not be computed.

use navigate_celestial::{AnalyticError, ObservationError};

/// A failed eclipse calculation.
#[derive(Debug, thiserror::Error)]
pub enum EclipseError {
    /// Navigate could not give the Sun or Moon at an instant, usually because it lies
    /// outside its 1850 to 2100 range.
    #[error("no {body} position at JD {jd_tt:.5} TT")]
    Ephemeris {
        /// "Sun" or "Moon".
        body: &'static str,
        /// The instant, as a TT Julian date.
        jd_tt: f64,
        /// Navigate's error.
        #[source]
        source: ObservationError<AnalyticError>,
    },
    /// An instant lies outside the sampled window.
    #[error("JD {jd_tt:.5} TT lies outside the sampled window {start:.5} to {end:.5}")]
    OutsideWindow {
        /// The instant asked for.
        jd_tt: f64,
        /// Window start.
        start: f64,
        /// Window end.
        end: f64,
    },
    /// No eclipse occurs at the syzygy asked for.
    #[error("no eclipse at the syzygy near JD {jd_tt:.3} TT")]
    NoEclipse {
        /// The syzygy.
        jd_tt: f64,
    },
}
