//! Kelmarsh Turbine_Data CSV parser.
//!
//! Reads Greenbyte-formatted SCADA exports and produces `CanonicalRow`
//! instances. See `SCHEMA.md` for the source-to-canonical mapping.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

/// Open a Kelmarsh Turbine_Data CSV and print its header row.
///
/// This is a scaffolding function to prove the file structure is
/// handled correctly:
///   - Opens the file
///   - Skips 9 comment lines
///   - Reads line 10 as the header
///   - Splits by comma (RFC-4180 aware via csv crate)
///   - Prints one column name per line, numbered
///
/// Not part of the final parser API — Piece 4 replaces this with
/// actual CanonicalRow extraction.
pub fn print_header(csv_path: &Path) -> anyhow::Result<()> {
    let file = File::open(csv_path)?;
    let mut reader = BufReader::new(file);

    // Skip the 9 comment lines at the top of Greenbyte exports.
    let mut discard = String::new();
    for _ in 0..9 {
        discard.clear();
        reader.read_line(&mut discard)?;
    }

    // Now hand the rest of the file to the csv crate to parse the header.
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();

    println!("Found {} columns:", headers.len());
    for (i, col) in headers.iter().enumerate() {
        println!("{:4}  {}", i + 1, col);
    }

    Ok(())
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