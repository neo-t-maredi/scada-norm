use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

use scada_norm::parser;
use scada_norm::writer;

#[derive(Parser)]
#[command(
    name = "scada-norm",
    version,
    about = "Normalize turbine SCADA exports to a canonical Parquet schema"
)]
struct Cli {
    /// Input SCADA CSV export
    input: PathBuf,

    /// Output Parquet path
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Read.
    let start = Instant::now();
    let (rows, stats) = parser::read_all_rows(&cli.input)?;
    let read_elapsed = start.elapsed();

    println!("Read {} rows in {:.2?}", stats.rows_ok, read_elapsed);
    println!("  rows_failed: {}", stats.rows_failed);

    // Write.
    if let Some(parent) = cli.output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let start = Instant::now();
    writer::write_canonical_rows_to_parquet(&rows, &cli.output)?;
    let write_elapsed = start.elapsed();

    println!("Wrote {} in {:.2?}", cli.output.display(), write_elapsed);

    Ok(())
}