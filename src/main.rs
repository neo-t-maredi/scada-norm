mod canonical;
mod parser;

use std::path::Path;

fn main() -> anyhow::Result<()> {
    let csv_path = Path::new(
        "data/raw/Turbine_Data_Kelmarsh_1_2020-01-01_-_2021-01-01_228.csv"
    );

    let row = parser::read_first_row(csv_path)?;

    println!("Successfully parsed first row:");
    println!("{:#?}", row);

    Ok(())
}