use scada_norm::parser;

use std::path::Path;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let csv_path = Path::new(
        "data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv"
    );

    let start = Instant::now();
    let (rows, stats) = parser::read_all_rows(csv_path)?;
    let elapsed = start.elapsed();

    println!("Parsed file in {:.2?}", elapsed);
    println!("  rows_ok:     {}", stats.rows_ok);
    println!("  rows_failed: {}", stats.rows_failed);

    if let Some(first) = rows.first() {
        println!("\nFirst row:");
        println!("{:#?}", first);
    }

    if let Some(last) = rows.last() {
        println!("\nLast row:");
        println!("{:#?}", last);
    }

    Ok(())
}