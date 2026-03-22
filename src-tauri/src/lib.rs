// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Brice LECOLE

pub mod commands;
pub mod file_assoc;
pub mod file_operations;
pub mod hex_parser;
pub mod srec_parser;

use std::sync::Mutex;
use tauri::{Emitter, Manager, RunEvent};

/// Holds the path of the file to open at startup (CLI arg on Windows/Linux,
/// or macOS open-file Apple Event received before the webview is ready).
pub struct StartupFile(pub Mutex<Option<String>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // On Windows/Linux the OS passes the associated file as argv[1]
    let startup_path = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-') && std::path::Path::new(a).exists());

    tauri::Builder::default()
        .manage(StartupFile(Mutex::new(startup_path)))
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
            commands::get_startup_file,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            // macOS sends RunEvent::Opened when the app is asked to open a file
            // via a file-association double-click (Apple Events / openFile:).
            if let RunEvent::Opened { urls } = event {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        if let Some(path_str) = path.to_str() {
                            // Store the path so the frontend can retrieve it
                            // on mount in case the webview wasn't ready yet.
                            if let Some(state) = app_handle.try_state::<StartupFile>() {
                                if let Ok(mut guard) = state.0.lock() {
                                    *guard = Some(path_str.to_string());
                                }
                            }
                            // Also emit directly for warm launches (app already open)
                            let _ = app_handle.emit("open-file", path_str);
                        }
                    }
                }
            }
        });
}
