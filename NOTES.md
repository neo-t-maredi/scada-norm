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
### README revision
- Current README is Day 1 status boilerplate
- Revisit on Day 6 with real shipped features to describe
- Add: canonical schema description, ops-engineer framing, example command

## Wednesday 2026-07-22 — Piece 7 shipped

### Delivered
- `src/writer.rs`: Arrow schema + row-to-column pivot + Parquet writer with Snappy compression
- End-to-end pipeline verified: Kelmarsh CSV (470MB, 299 cols) → CanonicalRow (11 cols) → Parquet (3.9MB)
- Independent verification via pyarrow: schema honored, timestamps UTC-tagged, full year present

### Performance
- Read 52,704 rows: ~2.3s (release build, cold cache)
- Write Parquet: 46ms
- On-disk compression: 20x (from ~78MB/turbine CSV to 3.9MB Parquet)

### Next session (Thursday)
1. Loop over all 6 turbines (Piece 6, deferred)
2. README rewrite — real content, not scaffold boilerplate
3. LinkedIn post draft
4. Ship to GitHub

### Do not touch
- File organization refactors (parser.rs split → after v1 ships)
- Data quality report (v1.1)
- Second dataset (v1.2)
