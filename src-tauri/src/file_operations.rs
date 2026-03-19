use std::fs;
use std::path::Path;

/// Read an entire file into memory, returning raw bytes.
pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("failed to read '{path}': {e}"))
}

/// Read a text file (UTF-8), returning the string content.
pub fn read_text_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read text file '{path}': {e}"))
}

/// Write raw bytes to a file, overwriting if it exists.
pub fn write_file(path: &str, data: &[u8]) -> Result<(), String> {
    fs::write(path, data).map_err(|e| format!("failed to write '{path}': {e}"))
}

/// Return a file's size in bytes without reading its contents.
pub fn file_size(path: &str) -> Result<u64, String> {
    let meta = fs::metadata(path).map_err(|e| format!("failed to stat '{path}': {e}"))?;
    Ok(meta.len())
}

/// Detect the file format based on extension and/or magic bytes.
/// Returns "ihex", "srec", "binary", or "unknown".
pub fn detect_format(path: &str) -> String {
    let p = Path::new(path);
    match p.extension().and_then(|e| e.to_str()) {
        Some("hex") | Some("ihex") => return "ihex".into(),
        Some("srec") | Some("mot") | Some("s19") | Some("s28") | Some("s37") => {
            return "srec".into()
        }
        Some("bin") => return "binary".into(),
        _ => {}
    }

    // Fall back to magic-byte sniffing (read first 2 bytes)
    if let Ok(mut f) = fs::File::open(path) {
        use std::io::Read;
        let mut buf = [0u8; 2];
        if f.read_exact(&mut buf).is_ok() {
            if buf[0] == b':' {
                return "ihex".into();
            }
            if buf[0] == b'S' && buf[1].is_ascii_digit() {
                return "srec".into();
            }
        }
    }

    "unknown".into()
}
