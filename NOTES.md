# NOTES

## Day 1 — 2026-07-15

### Environment
- Rust 1.97.0 via rustup, pinned in `rust-toolchain.toml`
- All deps compile clean, `cargo build` finishes in ~8s
- `cargo-audit` and `cargo-deny` installed
- VS Code with rust-analyzer, `crates`, Even Better TOML, Rainbow CSV

### Data
- Kelmarsh 2020 data at `data/raw/Kelmarsh_SCADA_2020_3086.zip`, unzipped
- 12 CSVs: 6 × `Turbine_Data_Kelmarsh_N_*.csv` + 6 × `Status_Kelmarsh_N_*.csv`
- Also have `Kelmarsh_WT_static.csv` — turbine specs (rated power, hub height, coords)

### Key discoveries
- Greenbyte export has 9 comment lines starting with `#`, header on line 10
- Time zone is UTC (stated in comments) — no DST parsing needed
- Missing/erroneous values are literal string "NaN"
- Turbine_Data files have ~230 columns per turbine — we only need ~13
- Status files have status codes and IEC categories — Greenbyte has partly
  normalized Senvion codes to IEC 61400-25 already
- Hub heights differ across turbines: T1/T2/T4/T5 at 78.5m, T3/T6 at 68.5m

### Tomorrow (Day 2)
1. Verify exact column names in Turbine_Data header (Ctrl+F, copy strings)
2. Write the CanonicalRow struct in Rust matching the 13-column schema
3. First code: open one CSV, skip 9 comment lines, parse header row,
   parse one data row, print it. Nothing more.

### Do not
- Try to parse all 230 columns
- Add a second dataset (Nørrekær) yet
- Touch Parquet output yet
- Skip lunch again