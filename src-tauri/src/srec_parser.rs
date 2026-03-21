// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Brice LECOLE

/// Parsed representation of a Motorola S-record file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SrecFile {
    pub records: Vec<SrecRecord>,
    pub total_data_bytes: usize,
}

/// A single S-record line.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SrecRecord {
    pub record_type: String, // e.g. "S0", "S1", "S2", "S3", "S5", "S7", "S8", "S9"
    pub address: u32,
    pub data: Vec<u8>,
}

/// Parse raw Motorola S-record text into a structured `SrecFile`.
///
/// Supports S0–S3 (header/data), S5 (record count), S7–S9 (end-of-block).
/// Returns an error string on malformed input.
pub fn parse(raw: &str) -> Result<SrecFile, String> {
    let mut records: Vec<SrecRecord> = Vec::new();
    let mut total_data_bytes: usize = 0;

    for (line_num, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('S') || line.len() < 4 {
            return Err(format!("line {}: invalid S-record: {line}", line_num + 1));
        }

        let record_type = &line[..2];
        let bytes = hex_decode(&line[2..])
            .map_err(|e| format!("line {}: {e}", line_num + 1))?;

        if bytes.is_empty() {
            return Err(format!("line {}: empty byte count", line_num + 1));
        }

        // Verify checksum (last byte = one's complement of sum of all preceding bytes)
        let data_bytes = &bytes[..bytes.len() - 1];
        let checksum_byte = bytes[bytes.len() - 1];
        let computed: u8 = data_bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        let expected = !computed;
        if expected != checksum_byte {
            return Err(format!(
                "line {}: checksum mismatch (got {checksum_byte:#04x}, expected {expected:#04x})",
                line_num + 1
            ));
        }

        // byte_count field is bytes[0]; remaining = address + data + checksum
        let addr_data_chk = &bytes[1..];

        let (address, data) = match record_type {
            // 2-byte address
            "S0" | "S1" | "S5" | "S9" => {
                if addr_data_chk.len() < 3 {
                    return Err(format!("line {}: record too short", line_num + 1));
                }
                let addr = u32::from(addr_data_chk[0]) << 8 | u32::from(addr_data_chk[1]);
                let data = addr_data_chk[2..addr_data_chk.len() - 1].to_vec();
                (addr, data)
            }
            // 3-byte address
            "S2" | "S8" => {
                if addr_data_chk.len() < 4 {
                    return Err(format!("line {}: record too short", line_num + 1));
                }
                let addr = u32::from(addr_data_chk[0]) << 16
                    | u32::from(addr_data_chk[1]) << 8
                    | u32::from(addr_data_chk[2]);
                let data = addr_data_chk[3..addr_data_chk.len() - 1].to_vec();
                (addr, data)
            }
            // 4-byte address
            "S3" | "S7" => {
                if addr_data_chk.len() < 5 {
                    return Err(format!("line {}: record too short", line_num + 1));
                }
                let addr = u32::from(addr_data_chk[0]) << 24
                    | u32::from(addr_data_chk[1]) << 16
                    | u32::from(addr_data_chk[2]) << 8
                    | u32::from(addr_data_chk[3]);
                let data = addr_data_chk[4..addr_data_chk.len() - 1].to_vec();
                (addr, data)
            }
            // S6 carries a 3-byte record count and is not part of the official
            // standard, but many tools emit it. Silently skip it.
            "S6" => continue,
            other => return Err(format!("line {}: unknown record type: {other}", line_num + 1)),
        };

        if record_type == "S1" || record_type == "S2" || record_type == "S3" {
            total_data_bytes += data.len();
        }

        records.push(SrecRecord {
            record_type: record_type.to_string(),
            address,
            data,
        });
    }

    Ok(SrecFile {
        records,
        total_data_bytes,
    })
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd-length hex string".into());
    }
    (0..s.len() / 2)
        .map(|i| {
            u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
                .map_err(|_| format!("invalid hex byte at offset {i}"))
        })
        .collect()
}
