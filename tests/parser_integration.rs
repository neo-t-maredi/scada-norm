//! Integration tests for the parser module.
//!
//! These tests exercise the full read path using a synthetic fixture
//! that mimics the Kelmarsh Greenbyte CSV format.

use scada_norm::parser::read_all_rows;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("fixtures");
    p.push(name);
    p
}

#[test]
fn reads_all_three_fixture_rows() {
    let path = fixture_path("Turbine_Data_Kelmarsh_1_fixture.csv");
    let (rows, stats) = read_all_rows(&path).expect("fixture should parse");

    assert_eq!(stats.rows_ok, 3, "expected 3 successful rows");
    assert_eq!(stats.rows_failed, 0, "expected zero failures");
    assert_eq!(rows.len(), 3);
}

#[test]
fn first_row_has_correct_canonical_fields() {
    let path = fixture_path("Turbine_Data_Kelmarsh_1_fixture.csv");
    let (rows, _) = read_all_rows(&path).expect("fixture should parse");

    let first = &rows[0];
    assert_eq!(first.turbine_id, "KWF1");
    assert_eq!(first.wind_speed_ms, Some(5.0));
    assert_eq!(first.wind_direction_deg, Some(180.0));
    assert_eq!(first.active_power_kw, Some(500.0));
    assert_eq!(first.pitch_angle_deg, Some(2.5));
}
