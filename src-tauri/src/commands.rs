// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Brice LECOLE

use crate::{file_operations::{self, RecordData}, hex_parser, srec_parser};

/// Read a file from the filesystem and return its raw bytes.
///
/// The Vec<u8> is serialized as a JSON array of integers by Tauri.
#[tauri::command]
pub fn open_file(path: String) -> Result<Vec<u8>, String> {
    file_operations::read_file(&path)
}

/// Parse an Intel HEX payload (raw bytes of a .hex file) and return a
/// JSON-serialized summary of the records.
#[tauri::command]
pub fn parse_intel_hex(data: Vec<u8>) -> Result<String, String> {
    let text =
        String::from_utf8(data).map_err(|e| format!("file is not valid UTF-8: {e}"))?;
    let hex_file = hex_parser::parse(&text)?;
    serde_json::to_string(&hex_file).map_err(|e| format!("serialization error: {e}"))
}

/// Parse a Motorola S-record payload and return a JSON-serialized summary.
#[tauri::command]
pub fn parse_srec(data: Vec<u8>) -> Result<String, String> {
    let text =
        String::from_utf8(data).map_err(|e| format!("file is not valid UTF-8: {e}"))?;
    let srec_file = srec_parser::parse(&text)?;
    serde_json::to_string(&srec_file).map_err(|e| format!("serialization error: {e}"))
}

/// Detect the file format ("ihex", "srec", "binary", or "unknown") from
/// the file extension and/or leading magic bytes.
#[tauri::command]
pub fn detect_file_format(path: String) -> String {
    file_operations::detect_format(&path)
}

/// Flatten records into a raw binary blob and write to disk.
/// Returns the number of bytes written.
#[tauri::command]
pub fn save_binary(records: Vec<RecordData>, path: String, fill_byte: u8) -> Result<u64, String> {
    file_operations::write_binary(&records, &path, fill_byte)
}

/// Serialise the current records to the requested format and write them to disk.
///
/// `format` must be `"ihex"` or `"srec"` (case-insensitive).
/// Only "Data" records in the input are written; all metadata records
/// (ExtendedLinearAddress, S0 header, etc.) are regenerated automatically.
#[tauri::command]
pub fn save_file(records: Vec<RecordData>, path: String, format: String) -> Result<(), String> {
    let content = match format.to_lowercase().as_str() {
        "ihex" => file_operations::write_ihex(&records),
        "srec" => file_operations::write_srec(&records),
        other  => return Err(format!("Unsupported format '{other}'. Use 'ihex' or 'srec'.")),
    };
    file_operations::write_file(&path, content.as_bytes())
}
