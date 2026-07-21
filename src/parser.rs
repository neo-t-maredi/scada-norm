//! Kelmarsh Turbine_Data CSV parser.
//!
//! Reads Greenbyte-formatted SCADA exports and produces `CanonicalRow`
//! instances. See `SCHEMA.md` for the source-to-canonical mapping.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::canonical::CanonicalRow;
use chrono::{DateTime, NaiveDateTime, Utc};

/// Statistics from parsing a Kelmarsh SCADA file.
#[derive(Debug, Default, Clone)]
pub struct ParseStats {
    /// Number of data rows successfully parsed into CanonicalRow.
    pub rows_ok: usize,
    /// Number of data rows that failed to parse (bad timestamp, malformed record, etc).
    pub rows_failed: usize,
}

// Column indices (0-based) into the Kelmarsh Turbine_Data CSV.
// See SCHEMA.md for the source-to-canonical mapping and docs/kelmarsh_columns.txt
// for the full column reference.
const COL_TIMESTAMP: usize = 0;
const COL_WIND_SPEED: usize = 1;
const COL_WIND_DIRECTION: usize = 15;
const COL_NACELLE_POSITION: usize = 16;
const COL_ACTIVE_POWER: usize = 61;
const COL_REACTIVE_POWER: usize = 86;
const COL_AMBIENT_TEMP: usize = 93;
const COL_ROTOR_RPM: usize = 210;
const COL_GENERATOR_RPM: usize = 211;
const COL_PITCH_ANGLE: usize = 245;

/// Parse a string field into an optional f64.
///
/// Returns `None` for the Greenbyte "NaN" sentinel and empty strings.
/// Returns `Some(x)` for any parseable floating-point number.
///
/// This is the single point where the SCADA export's string-encoded
/// missing values become proper Option<f64> nulls in the canonical form.
fn parse_optional_f64(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "NaN" {
        None
    } else {
        trimmed.parse::<f64>().ok()
    }
}

/// Extract the canonical turbine ID from a Kelmarsh filename.
///
/// Kelmarsh filenames follow the pattern:
///   `Turbine_Data_Kelmarsh_N_<date_range>.csv`
///
/// where N is the turbine number (1-6). This is mapped to the canonical
/// form `KWF{N}` per SCHEMA.md.
///
/// Returns `None` if the filename doesn't match the expected pattern.
fn turbine_id_from_path(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;

    // Split by underscore and find the turbine number.
    // Expected: ["Turbine", "Data", "Kelmarsh", "N", ...]
    let parts: Vec<&str> = filename.split('_').collect();

    if parts.len() < 4 {
        return None;
    }

    if parts[0] != "Turbine" || parts[1] != "Data" || parts[2] != "Kelmarsh" {
        return None;
    }

    let turbine_number: u8 = parts[3].parse().ok()?;

    if !(1..=6).contains(&turbine_number) {
        return None;
    }

    Some(format!("KWF{}", turbine_number))
}

/// Read the first data row from a Kelmarsh Turbine_Data CSV and return
/// it as a CanonicalRow.
///
/// This is Piece 4 of the parser walkthrough — proves that we can:
///   - Open the file
///   - Skip 9 comment lines
///   - Parse the header
///   - Read one data row
///   - Extract 10 canonical fields by column index
///   - Handle NaN sentinels via parse_optional_f64
///   - Attach turbine_id from the filename
///   - Parse the timestamp as UTC


/// Read all data rows from a Kelmarsh Turbine_Data CSV file.
///
/// Row-level failures (malformed timestamp, unparseable field, missing column)
/// are counted in `ParseStats` and the row is skipped. File-level failures
/// (file not found, missing header) return `Err`.
pub fn read_all_rows(csv_path: &Path) -> anyhow::Result<(Vec<CanonicalRow>, ParseStats)> {
    let turbine_id = turbine_id_from_path(csv_path)
        .ok_or_else(|| anyhow::anyhow!("could not extract turbine ID from filename"))?;

    let file = File::open(csv_path)?;
    let mut reader = BufReader::new(file);

    // Skip 9 comment lines.
    let mut discard = String::new();
    for _ in 0..9 {
        discard.clear();
        reader.read_line(&mut discard)?;
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let mut rows = Vec::new();
    let mut stats = ParseStats::default();

    for result in csv_reader.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                stats.rows_failed += 1;
                continue;
            }
        };

        match parse_row(&record, &turbine_id) {
            Some(row) => {
                rows.push(row);
                stats.rows_ok += 1;
            }
            None => {
                stats.rows_failed += 1;
            }
        }
    }

    Ok((rows, stats))
}

/// Extract a CanonicalRow from a single CSV record.
///
/// Returns None on any parse failure (bad timestamp, missing column, etc).
/// This is the single point where row-level failures are converted to `None`
/// for the skip-and-count strategy in `read_all_rows`.
fn parse_row(record: &csv::StringRecord, turbine_id: &str) -> Option<CanonicalRow> {
    let ts_str = record.get(COL_TIMESTAMP)?;
    let naive = NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let timestamp_utc = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);

    let get_f64 = |idx: usize| -> Option<f64> {
        record.get(idx).and_then(parse_optional_f64_str)
    };

    Some(CanonicalRow {
        timestamp_utc,
        turbine_id: turbine_id.to_string(),
        wind_speed_ms: get_f64(COL_WIND_SPEED),
        wind_direction_deg: get_f64(COL_WIND_DIRECTION),
        nacelle_position_deg: get_f64(COL_NACELLE_POSITION),
        active_power_kw: get_f64(COL_ACTIVE_POWER),
        reactive_power_kvar: get_f64(COL_REACTIVE_POWER),
        ambient_temp_c: get_f64(COL_AMBIENT_TEMP),
        rotor_rpm: get_f64(COL_ROTOR_RPM),
        generator_rpm: get_f64(COL_GENERATOR_RPM),
        pitch_angle_deg: get_f64(COL_PITCH_ANGLE),
    })
}

/// Adapter: `parse_optional_f64` takes `&str`, but our closure receives
/// `&str` from record.get(). This wrapper matches the closure signature
/// expected by Option's `and_then`.
fn parse_optional_f64_str(s: &str) -> Option<f64> {
    parse_optional_f64(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_number() {
        assert_eq!(parse_optional_f64("3.87"), Some(3.87));
    }

    #[test]
    fn nan_sentinel_becomes_none() {
        assert_eq!(parse_optional_f64("NaN"), None);
    }

    #[test]
    fn empty_string_becomes_none() {
        assert_eq!(parse_optional_f64(""), None);
    }

    #[test]
    fn whitespace_only_becomes_none() {
        assert_eq!(parse_optional_f64("   "), None);
    }

    #[test]
    fn unparseable_becomes_none() {
        assert_eq!(parse_optional_f64("hello"), None);
    }

    #[test]
    fn extracts_kelmarsh_1() {
        let path = Path::new("data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv");
        assert_eq!(turbine_id_from_path(path), Some("KWF1".to_string()));
    }

    #[test]
    fn extracts_kelmarsh_6() {
        let path = Path::new("Turbine_Data_Kelmarsh_6_2020-01-01_-_2021-01-01_233.csv");
        assert_eq!(turbine_id_from_path(path), Some("KWF6".to_string()));
    }

    #[test]
    fn rejects_status_file() {
        let path = Path::new("Status_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv");
        assert_eq!(turbine_id_from_path(path), None);
    }

    #[test]
    fn rejects_out_of_range_turbine() {
        let path = Path::new("Turbine_Data_Kelmarsh_99_something.csv");
        assert_eq!(turbine_id_from_path(path), None);
    }

    #[test]
    fn rejects_garbage_filename() {
        let path = Path::new("random.csv");
        assert_eq!(turbine_id_from_path(path), None);
    }
}