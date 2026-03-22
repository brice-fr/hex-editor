// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Brice LECOLE

pub mod commands;
pub mod file_assoc;
pub mod file_operations;
pub mod hex_parser;
pub mod srec_parser;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::open_file,
            commands::parse_intel_hex,
            commands::parse_srec,
            commands::detect_file_format,
            commands::save_file,
            commands::save_binary,
            commands::get_file_associations,
            commands::apply_file_associations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
