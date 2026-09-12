use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;

use scada_norm::batch;

#[derive(Parser)]
#[command(
    name = "scada-norm",
    version,
    about = "Normalize turbine SCADA exports to a canonical Parquet schema"
)]
struct Cli {
    /// Input SCADA CSV export, or a directory containing them
    input: PathBuf,

    /// Output Parquet path, or output directory when INPUT is a directory
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.input.is_dir() {
        let report = batch::normalize_dir(&cli.input, &cli.output)?;
        println!(
            "Batch complete: {} files, {} rows, {} failed",
            report.files, report.rows_ok, report.failed
        );
        if report.failed > 0 {
            bail!("{} file(s) failed", report.failed);
        }
        Ok(())
    } else {
        batch::normalize_one(&cli.input, &cli.output).map(|_| ())
    }
}
