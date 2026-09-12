//! Parquet writer for CanonicalRow batches.

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, StringArray, TimestampNanosecondArray};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use crate::canonical::CanonicalRow;

fn canonical_schema() -> Schema {
    Schema::new(vec![
        Field::new(
            "timestamp_utc",
            DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
            false,
        ),
        Field::new("turbine_id", DataType::Utf8, false),
        Field::new("wind_speed_ms", DataType::Float64, true),
        Field::new("wind_direction_deg", DataType::Float64, true),
        Field::new("nacelle_position_deg", DataType::Float64, true),
        Field::new("active_power_kw", DataType::Float64, true),
        Field::new("reactive_power_kvar", DataType::Float64, true),
        Field::new("ambient_temp_c", DataType::Float64, true),
        Field::new("rotor_rpm", DataType::Float64, true),
        Field::new("generator_rpm", DataType::Float64, true),
        Field::new("pitch_angle_deg", DataType::Float64, true),
    ])
}

fn rows_to_record_batch(rows: &[CanonicalRow]) -> anyhow::Result<RecordBatch> {
    let schema = Arc::new(canonical_schema());

    let ts_values: Vec<i64> = rows
        .iter()
        .map(|r| r.timestamp_utc.timestamp_nanos_opt().unwrap_or(0))
        .collect();
    let ts_array = TimestampNanosecondArray::from(ts_values).with_timezone("UTC");

    let turbine_ids: Vec<&str> = rows.iter().map(|r| r.turbine_id.as_str()).collect();
    let turbine_id_array = StringArray::from(turbine_ids);

    let collect_f64 = |extract: fn(&CanonicalRow) -> Option<f64>| -> Float64Array {
        Float64Array::from_iter(rows.iter().map(extract))
    };

    let wind_speed = collect_f64(|r| r.wind_speed_ms);
    let wind_direction = collect_f64(|r| r.wind_direction_deg);
    let nacelle_position = collect_f64(|r| r.nacelle_position_deg);
    let active_power = collect_f64(|r| r.active_power_kw);
    let reactive_power = collect_f64(|r| r.reactive_power_kvar);
    let ambient_temp = collect_f64(|r| r.ambient_temp_c);
    let rotor_rpm = collect_f64(|r| r.rotor_rpm);
    let generator_rpm = collect_f64(|r| r.generator_rpm);
    let pitch_angle = collect_f64(|r| r.pitch_angle_deg);

    let columns: Vec<ArrayRef> = vec![
        Arc::new(ts_array),
        Arc::new(turbine_id_array),
        Arc::new(wind_speed),
        Arc::new(wind_direction),
        Arc::new(nacelle_position),
        Arc::new(active_power),
        Arc::new(reactive_power),
        Arc::new(ambient_temp),
        Arc::new(rotor_rpm),
        Arc::new(generator_rpm),
        Arc::new(pitch_angle),
    ];

    let batch = RecordBatch::try_new(schema, columns)?;
    Ok(batch)
}

pub fn write_canonical_rows_to_parquet(
    rows: &[CanonicalRow],
    output_path: &Path,
) -> anyhow::Result<()> {
    if rows.is_empty() {
        anyhow::bail!("cannot write empty row set to Parquet");
    }

    let batch = rows_to_record_batch(rows)?;
    let schema = batch.schema();

    let file = File::create(output_path)?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(())
}
