//! Batch normalization.
//!
//! `normalize_one` converts a single SCADA CSV export to Parquet.
//! `normalize_dir` applies it to every `.csv` in a directory, naming each
//! output after the turbine ID derived from the input filename.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, bail};

use crate::parser::{self, ParseStats};
use crate::writer;

/// Summary of a directory run. Per-file failures are counted, not fatal,
/// so one bad export does not block the rest of the fleet.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BatchReport {
    pub files: usize,
    pub rows_ok: usize,
    pub failed: usize,
}

/// Normalize one CSV export to one Parquet file.
pub fn normalize_one(input: &Path, output: &Path) -> anyhow::Result<ParseStats> {
    let start = Instant::now();
    let (rows, stats) = parser::read_all_rows(input)?;
    println!("Read {} rows in {:.2?}", stats.rows_ok, start.elapsed());
    println!("  rows_failed: {}", stats.rows_failed);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let start = Instant::now();
    writer::write_canonical_rows_to_parquet(&rows, output)?;
    println!("Wrote {} in {:.2?}", output.display(), start.elapsed());

    Ok(stats)
}

/// Normalize every `.csv` in `input_dir` into `output_dir`.
///
/// Returns `Err` only if the directory cannot be read or contains no CSV
/// files. Files whose name does not yield a turbine ID, or which fail to
/// parse, are reported in `BatchReport::failed` and skipped.
pub fn normalize_dir(input_dir: &Path, output_dir: &Path) -> anyhow::Result<BatchReport> {
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(input_dir)
        .with_context(|| format!("reading {}", input_dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
        })
        .collect();
    inputs.sort();

    if inputs.is_empty() {
        bail!("no .csv files found in {}", input_dir.display());
    }
    std::fs::create_dir_all(output_dir)?;

    let mut report = BatchReport {
        files: inputs.len(),
        ..Default::default()
    };

    for input in &inputs {
        let name = input.file_name().unwrap_or_default().to_string_lossy();

        let Some(turbine_id) = parser::turbine_id_from_path(input) else {
            eprintln!("SKIP {name}: could not derive turbine_id from filename");
            report.failed += 1;
            continue;
        };

        println!("=== {turbine_id} ===");
        let output = output_dir.join(format!("{}.parquet", turbine_id.to_lowercase()));

        match normalize_one(input, &output) {
            Ok(stats) => report.rows_ok += stats.rows_ok,
            Err(err) => {
                eprintln!("FAIL {name}: {err:#}");
                report.failed += 1;
            }
        }
    }

    Ok(report)
}
