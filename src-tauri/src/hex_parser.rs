use ihex::Record;

/// Parsed representation of an Intel HEX file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HexFile {
    pub records: Vec<HexRecord>,
    pub total_data_bytes: usize,
}

/// A single record extracted from an Intel HEX file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HexRecord {
    pub record_type: String,
    pub address: u32,
    pub data: Vec<u8>,
}

/// Parse raw Intel HEX text into a structured `HexFile`.
///
/// Returns an error string if the input is malformed.
pub fn parse(raw: &str) -> Result<HexFile, String> {
    let reader = ihex::Reader::new(raw);

    let mut records: Vec<HexRecord> = Vec::new();
    let mut total_data_bytes: usize = 0;
    let mut extended_linear_addr: u32 = 0;

    for result in reader {
        let record = result.map_err(|e| format!("ihex parse error: {e}"))?;

        match &record {
            Record::ExtendedLinearAddress(upper) => {
                extended_linear_addr = (*upper as u32) << 16;
                records.push(HexRecord {
                    record_type: "ExtendedLinearAddress".into(),
                    address: extended_linear_addr,
                    data: vec![],
                });
            }
            Record::Data { offset, value } => {
                total_data_bytes += value.len();
                records.push(HexRecord {
                    record_type: "Data".into(),
                    address: extended_linear_addr | (*offset as u32),
                    data: value.clone(),
                });
            }
            Record::EndOfFile => {
                records.push(HexRecord {
                    record_type: "EndOfFile".into(),
                    address: 0,
                    data: vec![],
                });
            }
            Record::ExtendedSegmentAddress(seg) => {
                records.push(HexRecord {
                    record_type: "ExtendedSegmentAddress".into(),
                    address: (*seg as u32) << 4,
                    data: vec![],
                });
            }
            Record::StartSegmentAddress { cs, ip } => {
                records.push(HexRecord {
                    record_type: "StartSegmentAddress".into(),
                    address: ((*cs as u32) << 4) + *ip as u32,
                    data: vec![],
                });
            }
            Record::StartLinearAddress(addr) => {
                records.push(HexRecord {
                    record_type: "StartLinearAddress".into(),
                    address: *addr,
                    data: vec![],
                });
            }
        }
    }

    Ok(HexFile {
        records,
        total_data_bytes,
    })
}
