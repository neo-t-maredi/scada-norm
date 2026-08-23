scada-norm

Normalize turbine SCADA exports into a canonical Parquet schema.

Every SCADA historian exports differently. Column names, units, missing-value encodings, and timestamp conventions vary by vendor, by site, and sometimes by export job. Analysis code written against one export breaks against the next, and the fix usually ends up as a one-off script that lives in someone's home directory.

scada-norm pins vendor exports to a fixed 11-column canonical schema written to Parquet, so downstream work — power curves, availability, loss accounting — targets one stable contract instead of whatever the historian produced that day.

Quickstart
scada-norm <INPUT> --output <OUTPUT>
$ scada-norm data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv \
    -o data/normalized/kwf1_2020.parquet

Read 52704 rows in 615.96ms
  rows_failed: 0
Wrote data/normalized/kwf1_2020.parquet in 74.64ms

Build from source:

git clone https://github.com/neo-t-maredi/scada-norm
cd scada-norm
cargo build --release
Canonical schema
Column	Type	Unit	Description
timestamp_utc	datetime	UTC	10-min interval start timestamp
turbine_id	string	—	Canonical turbine identifier (e.g. KWF1)
wind_speed_ms	f64	m/s	Nacelle anemometer wind speed, 10-min mean
wind_direction_deg	f64	deg	Wind direction, 10-min mean
nacelle_position_deg	f64	deg	Nacelle yaw position, 10-min mean
active_power_kw	f64	kW	Active power output, 10-min mean
reactive_power_kvar	f64	kvar	Reactive power, 10-min mean
ambient_temp_c	f64	°C	Nacelle ambient temperature, 10-min mean
rotor_rpm	f64	RPM	Rotor rotational speed, 10-min mean
generator_rpm	f64	RPM	Generator rotational speed, 10-min mean
pitch_angle_deg	f64	deg	Blade A pitch angle, 10-min mean

Missing values become Parquet nulls, never sentinel values. Source-to-canonical column mappings and export quirks are documented in SCHEMA.md.

Verified run

Kelmarsh Wind Farm, turbine 1, calendar year 2020 (Greenbyte CSV export):

52,704 rows, 0 failed — exactly 366 × 144, so every 10-minute interval in the leap year is present with no gaps
UTC timestamps preserved end to end; no local-time round-tripping
299 source columns → 11 canonical
~78 MB CSV → 3.9 MB Parquet. Roughly 20×, but that figure is column pruning and Parquet's columnar layout and Snappy — not compression alone. The honest read is that most analysis never needed the other 288 columns.
Source support
Source	Format	Status
Kelmarsh / Greenbyte	CSV export	Supported

The Kelmarsh export has a 9-line comment preamble, a header on line 10 that itself begins with # , and NaN string literals for missing data. Parsing that correctly is most of what the v1 reader does. Adding a second vendor means a new source mapping, not a new schema.

Design notes
Parquet over CSV because the downstream consumers are columnar reads — pulling wind speed and power for a year shouldn't mean scanning 299 columns.
The schema is fixed, not inferred. Inferred schemas drift silently between exports, which is the exact failure this tool exists to prevent.
Nulls over sentinels. NaN, -9999, and empty string all mean "no data" in some export somewhere; they should not survive into analysis.
Library and binary are separate. main.rs handles argv and nothing else; the crate is usable without the CLI.
Out of scope for v1

Resampling, gap-filling, unit conversion beyond the canonical set, multi-turbine batch runs, and any form of data quality flagging. Normalization only — filtering and correction belong to the analysis layer, where the assumptions are visible.

License

MIT or Apache 2.0