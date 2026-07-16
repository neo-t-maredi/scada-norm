# scada-norm canonical schema (v1)

The canonical schema is the target format that all vendor SCADA exports
are normalized into. For v1, we support one source (Greenbyte CSV
exports from Kelmarsh) and one output (Parquet).

## Columns

| Canonical name | Type | Unit | Description |
|---|---|---|---|
| `timestamp_utc` | datetime | UTC | 10-min interval start timestamp |
| `turbine_id` | string | — | Canonical turbine identifier (e.g. `KWF1`) |
| `wind_speed_ms` | f64 | m/s | Nacelle-mounted anemometer wind speed, 10-min mean |
| `wind_direction_deg` | f64 | deg | Wind direction, 10-min mean |
| `nacelle_position_deg` | f64 | deg | Nacelle yaw position, 10-min mean |
| `active_power_kw` | f64 | kW | Active power output, 10-min mean |
| `reactive_power_kvar` | f64 | kvar | Reactive power, 10-min mean |
| `ambient_temp_c` | f64 | °C | Nacelle ambient temperature, 10-min mean |
| `rotor_rpm` | f64 | RPM | Rotor rotational speed, 10-min mean |
| `generator_rpm` | f64 | RPM | Generator rotational speed, 10-min mean |
| `pitch_angle_deg` | f64 | deg | Blade A pitch angle, 10-min mean |

Missing values are represented as Parquet nulls (not sentinel values).

## Source mapping — Kelmarsh (Greenbyte CSV)

Kelmarsh Turbine_Data CSV exports have 299 columns per row. The
canonical schema is a curated 11-column subset. Full column reference
in `docs/kelmarsh_columns.txt`.

| Canonical | Source column | Source column index |
|---|---|---|
| `timestamp_utc` | `# Date and time` | 1 |
| `wind_speed_ms` | `Wind speed (m/s)` | 2 |
| `wind_direction_deg` | `Wind direction (°)` | 16 |
| `nacelle_position_deg` | `Nacelle position (°)` | 17 |
| `active_power_kw` | `Power (kW)` | 62 |
| `reactive_power_kvar` | `Reactive power (kvar)` | 87 |
| `ambient_temp_c` | `Nacelle ambient temperature (°C)` | 94 |
| `rotor_rpm` | `Rotor speed (RPM)` | 211 |
| `generator_rpm` | `Generator RPM (RPM)` | 212 |
| `pitch_angle_deg` | `Blade angle (pitch position) A (°)` | 246 |

`turbine_id` is derived from the filename pattern
`Turbine_Data_Kelmarsh_N_*.csv` where `N` is the turbine number.
This maps to `KWF{N}` in canonical form.

## Kelmarsh CSV export quirks

- 9 comment lines at the top starting with `#`; header is on line 10
- Header line itself begins with `# ` making the first column literally
  named `# Date and time`
- Missing/erroneous values encoded as the literal string `NaN`
- Timestamps are UTC (declared in comment block), no timezone suffix
  on individual rows
- 10-minute intervals aligned to `:00`, `:10`, `:20`, `:30`, `:40`, `:50`