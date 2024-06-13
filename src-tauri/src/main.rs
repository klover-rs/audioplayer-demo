// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use tauri::App;

use crate::song_dir::{set_song_dir, get_songs_dir, get_songs};
use crate::audio_controls::get_current_index;

mod audio_backend;
mod event_handler;
mod audio_controls;
mod util;
mod song_dir;


fn main() {
    tauri::Builder::default()
        .setup(|app: &mut App| {
            let app_handle: tauri::AppHandle = app.handle();

            tauri::async_runtime::spawn(async move {
                event_handler::event_handler(app_handle).await
            });

            Ok(())
            
        })
        .invoke_handler(tauri::generate_handler![set_song_dir, get_songs_dir, get_songs, get_current_index])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
