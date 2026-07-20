//! Canonical data model.
//!
//! Defines the single, unified schema that all vendor-specific SCADA
//! exports are normalized into. Every source parser produces
//! `CanonicalRow` instances; every downstream tool consumes them.
//!
//! See `SCHEMA.md` at repo root for the schema specification and
//! source-to-canonical mapping.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single 10-minute SCADA reading in canonical form.
///
/// All measurements are 10-minute means unless otherwise noted.
/// Missing values are represented as `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRow {
    /// Timestamp of the 10-minute interval start (UTC).
    pub timestamp_utc: DateTime<Utc>,

    /// Canonical turbine identifier (e.g. "KWF1" for Kelmarsh turbine 1).
    pub turbine_id: String,

    /// Nacelle-mounted anemometer wind speed [m/s].
    pub wind_speed_ms: Option<f64>,

    /// Wind direction [degrees, 0-360].
    pub wind_direction_deg: Option<f64>,

    /// Nacelle yaw position [degrees, 0-360].
    pub nacelle_position_deg: Option<f64>,

    /// Active power output [kW].
    pub active_power_kw: Option<f64>,

    /// Reactive power [kvar].
    pub reactive_power_kvar: Option<f64>,

    /// Nacelle ambient temperature [°C].
    pub ambient_temp_c: Option<f64>,

    /// Rotor rotational speed [RPM].
    pub rotor_rpm: Option<f64>,

    /// Generator rotational speed [RPM].
    pub generator_rpm: Option<f64>,

    /// Blade A pitch angle [degrees].
    pub pitch_angle_deg: Option<f64>,
}