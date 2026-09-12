//! Integration tests for batch (directory) normalization.
//!
//! Builds a scratch input directory from the single fixture, runs
//! `normalize_dir` against it, and checks the report and output files.

use scada_norm::batch::{BatchReport, normalize_dir};
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests");
    p.push("fixtures");
    p.push(name);
    p
}

/// Fresh scratch directory per test so runs never collide.
fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scada-norm-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("raw")).expect("create scratch dir");
    dir
}

#[test]
fn normalizes_every_csv_in_directory() {
    let dir = scratch_dir("ok");
    let raw = dir.join("raw");
    let out = dir.join("normalized");
    let fixture = fixture_path("Turbine_Data_Kelmarsh_1_fixture.csv");

    fs::copy(&fixture, raw.join("Turbine_Data_Kelmarsh_1_fixture.csv")).unwrap();
    fs::copy(&fixture, raw.join("Turbine_Data_Kelmarsh_2_fixture.csv")).unwrap();
    fs::write(raw.join("notes.txt"), "not a csv, must be ignored").unwrap();

    let report = normalize_dir(&raw, &out).expect("batch should succeed");

    assert_eq!(
        report,
        BatchReport {
            files: 2,
            rows_ok: 6,
            failed: 0
        }
    );
    assert!(out.join("kwf1.parquet").is_file());
    assert!(out.join("kwf2.parquet").is_file());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unrecognized_filename_is_counted_not_fatal() {
    let dir = scratch_dir("skip");
    let raw = dir.join("raw");
    let out = dir.join("normalized");
    let fixture = fixture_path("Turbine_Data_Kelmarsh_1_fixture.csv");

    fs::copy(&fixture, raw.join("Turbine_Data_Kelmarsh_1_fixture.csv")).unwrap();
    fs::copy(&fixture, raw.join("mystery_export.csv")).unwrap();

    let report = normalize_dir(&raw, &out).expect("batch should still return a report");

    assert_eq!(report.files, 2);
    assert_eq!(report.rows_ok, 3);
    assert_eq!(report.failed, 1);
    assert!(out.join("kwf1.parquet").is_file());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn empty_directory_is_an_error() {
    let dir = scratch_dir("empty");
    let raw = dir.join("raw");

    assert!(normalize_dir(&raw, &dir.join("normalized")).is_err());

    let _ = fs::remove_dir_all(&dir);
}
