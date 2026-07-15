# Scada-Norm

A CLI for normalizing wind farm SCADA exports across aggregation systems.

Built by a SCADA engineer moving from oil & gas into wind, because the
multi-source SCADA consolidation problem shows up in every wind portfolio.

## Status

Day 1 (2026-07-15): Toolchain and scaffold set up. Real Kelmarsh data
downloaded and inspected. Schema design in progress.

## What it does (planned v1)

Reads Greenbyte-formatted SCADA CSV exports from the Kelmarsh wind farm
(6× Senvion MM92, Northamptonshire UK, 2020 data year) and emits a
normalized Parquet dataset with a small canonical schema suitable for
downstream power curve analysis, availability calculations, and
performance benchmarking.

## Roadmap

See `ROADMAP.md`.

## License

MIT OR Apache-2.0