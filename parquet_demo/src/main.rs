use parquet::{
	data_type,
	file::{properties::WriterProperties, writer::SerializedFileWriter},
	schema::{parser, printer},
};
use std::{path::Path, sync::Arc};

const MY_SCHEMA: &str = "
message my_message {
    required int64 receive_time (TIMESTAMP(NANOS, false));
    required byte_array name (string);
    required int64 id;
}
";

fn main() {
	// Create and dump schema.
	let my_schema_type =
		Arc::new(parser::parse_message_type(MY_SCHEMA).expect("Failed to parse schema"));
	printer::print_schema(&mut std::io::stdout(), &my_schema_type);

	// Create file and Parquet writer
	let out_path = Path::new("/tmp/test.parquet");
	let props = Arc::new(WriterProperties::new());
	let pq_file = std::fs::File::create(out_path).expect("Failed to create out file.");
	let mut pq_writer = SerializedFileWriter::new(pq_file, my_schema_type, props).unwrap();

	// Write rows/columns
	let mut row = pq_writer.next_row_group().unwrap();
	const ROW_COUNT: usize = 20;

	// Write timestamp
	if let Ok(Some(mut col1)) = row.next_column() {
		let writer = col1.typed::<data_type::Int64Type>();
		let dat: [i64; ROW_COUNT] = core::array::from_fn(|i| i as i64);
		writer
			.write_batch(&dat[..], None, None)
			.expect("Timestamp write failed");

		col1.close().expect("Col1 close failed");
	}

	// Write names
	if let Ok(Some(mut col1)) = row.next_column() {
		let writer = col1.typed::<data_type::ByteArrayType>();
		let dat: [String; ROW_COUNT] = core::array::from_fn(|i| i.to_string());
		let strs: Vec<data_type::ByteArray> = dat
			.iter()
			.map(|s| data_type::ByteArray::from(s.as_bytes()))
			.collect();
		writer
			.write_batch(&strs[..], None, None)
			.expect("Timestamp write failed");

		col1.close().expect("Col1 close failed");
	}

	// Write ids
	if let Ok(Some(mut col1)) = row.next_column() {
		let writer = col1.typed::<data_type::Int64Type>();
		let dat: [i64; ROW_COUNT] = core::array::from_fn(|i| 2 * i as i64);
		writer
			.write_batch(&dat[..], None, None)
			.expect("Timestamp write failed");

		col1.close().expect("Col1 close failed");
	}

	row.close().unwrap();

	// Finalize file and dump metadata
	let meta = pq_writer.close().expect("Failed to meta");
	println!("Out file meta: {meta:?}");
}
