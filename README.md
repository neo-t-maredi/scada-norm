# scada-norm

Normalize wind turbine SCADA exports into one fixed Parquet schema.

## The problem

A wind farm's control system (SCADA) records measurements from every turbine every ten minutes: wind speed, power output, rotor speed, blade angle, temperatures, and hundreds more. Operators export this data as large CSV files, one per turbine, often with several hundred columns.

Every vendor and every site exports differently. Column names, units, how missing values are written, and how timestamps are formatted all vary. Analysis code written against one export breaks on the next, and the fix usually becomes a one-off script in someone's home directory.

## What scada-norm does

scada-norm reads a vendor export and writes a compact Parquet file with eleven fixed columns. Downstream work such as power curves, availability, and loss accounting then targets one stable contract instead of whatever the historian produced that day.

It handles one file or a whole directory of them. Missing values become proper nulls, timestamps stay in UTC, and the output is roughly fifty times smaller than the input.

## Quickstart

Build from source (requires a Rust toolchain):

```
git clone https://github.com/neo-t-maredi/scada-norm
cd scada-norm
cargo build --release
```

Normalize a single export:

```
$ scada-norm data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv -o data/normalized/kwf1.parquet
Read 52704 rows in 615.96ms
  rows_failed: 0
Wrote data/normalized/kwf1.parquet in 74.64ms
```

Normalize every export in a directory. Each output is named after the turbine ID derived from the input filename:

```
$ scada-norm data/raw -o data/normalized
=== KWF1 ===
Read 52704 rows in 2.49s
  rows_failed: 0
Wrote data/normalized/kwf1.parquet in 170.04ms
...
Batch complete: 6 files, 316224 rows, 0 failed
```

A file that fails to parse, or whose name does not yield a turbine ID, is reported and skipped. The remaining files are still written, and the process exits non-zero so a script can notice.

## Getting the data

The tool is built and tested against the Kelmarsh Wind Farm open dataset (six Senvion MM92 turbines, Northamptonshire, UK). The 2020 exports used here are on Zenodo, record [16807551](https://zenodo.org/records/16807551). Download `Kelmarsh_SCADA_2020_3086.zip` and extract the `Turbine_Data_*.csv` files into `data/raw/`. The `Status_*.csv` files in the same archive are alarm logs and are not used.

## Output schema

| Column | Type | Unit | Description |
|---|---|---|---|
| `timestamp_utc` | datetime | UTC | Start of the 10-minute interval |
| `turbine_id` | string | — | Canonical turbine identifier, e.g. `KWF1` |
| `wind_speed_ms` | f64 | m/s | Nacelle anemometer wind speed, 10-min mean |
| `wind_direction_deg` | f64 | deg | Wind direction, 10-min mean |
| `nacelle_position_deg` | f64 | deg | Nacelle yaw position, 10-min mean |
| `active_power_kw` | f64 | kW | Active power output, 10-min mean |
| `reactive_power_kvar` | f64 | kvar | Reactive power, 10-min mean |
| `ambient_temp_c` | f64 | °C | Nacelle ambient temperature, 10-min mean |
| `rotor_rpm` | f64 | RPM | Rotor speed, 10-min mean |
| `generator_rpm` | f64 | RPM | Generator speed, 10-min mean |
| `pitch_angle_deg` | f64 | deg | Blade A pitch angle, 10-min mean |

Missing values are written as Parquet nulls, never as sentinel numbers. The mapping from each source column to the schema above, and the export quirks it works around, are documented in [SCHEMA.md](SCHEMA.md).

## Verified run

Kelmarsh Wind Farm, all six turbines, calendar year 2020 (Greenbyte CSV export):

- 316,224 rows in, 0 failed. Each turbine yields exactly 52,704 rows, which is 366 × 144: every 10-minute interval of the leap year, with no gaps.
- UTC timestamps preserved end to end, with no local-time round-tripping.
- 299 source columns reduced to 11.
- Six files totalling 1.2 GB of CSV become 24 MB of Parquet, in about 11 seconds.
- Maximum active power lands between 2,075 and 2,083 kW on all six turbines, consistent with the MM92's 2,050 kW rating in cold, dense air. If a column mapping had shifted on any file, that number would not cluster.

The size reduction is mostly column pruning, then Parquet's columnar layout, then Snappy compression. Most analysis never needed the other 288 columns.

## Supported sources

| Source | Format | Status |
|---|---|---|
| Kelmarsh / Greenbyte | CSV export | Supported |

The Kelmarsh export has a nine-line comment preamble, a header on line ten that itself begins with `#`, and the string `NaN` for missing data. Parsing that correctly is most of what the reader does. Adding a second vendor means a new source mapping, not a new schema.

## Design decisions

**Parquet, not CSV.** Downstream consumers read columns, not rows. Pulling a year of wind speed and power should not mean scanning 299 columns.

**Fixed schema, not inferred.** Inferred schemas drift silently between exports. That is the failure this tool exists to prevent.

**Nulls, not sentinels.** `NaN`, `-9999`, and empty strings all mean "no data" somewhere. None of them should survive into analysis.

**Per-file failure in batch mode.** One bad export should not block the other five. Failures are counted and reported; the exit code carries the verdict.

**Library and binary are separate.** `main.rs` handles arguments and nothing else. The crate is usable without the CLI.

## Not in scope

Resampling, gap-filling, unit conversion beyond the canonical set, and any form of data quality flagging. This tool normalizes. Filtering and correction belong to the analysis layer, where the assumptions are visible.

## License

See [LICENSE](LICENSE).
