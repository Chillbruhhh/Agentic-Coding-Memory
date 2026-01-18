#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod amp_client;
mod commands;

use commands::{get_amp_data, query_amp_objects};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_amp_data,
            query_amp_objects
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
