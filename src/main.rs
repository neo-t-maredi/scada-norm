use std::path::Path;
use std::time::Instant;

use scada_norm::parser;
use scada_norm::writer;

fn main() -> anyhow::Result<()> {
    let csv_path = Path::new(
        "data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv"
    );

    // Read.
    let start = Instant::now();
    let (rows, stats) = parser::read_all_rows(csv_path)?;
    let read_elapsed = start.elapsed();

    println!("Read {} rows in {:.2?}", stats.rows_ok, read_elapsed);
    println!("  rows_failed: {}", stats.rows_failed);

    // Write.
    let output_path = Path::new("data/normalized/kwf1_2020.parquet");
    let start = Instant::now();
    writer::write_canonical_rows_to_parquet(&rows, output_path)?;
    let write_elapsed = start.elapsed();

    println!("Wrote {} in {:.2?}", output_path.display(), write_elapsed);

    Ok(())
}
