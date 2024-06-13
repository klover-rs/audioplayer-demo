
use std::{fs, path::Path};

use crate::util::lmdb::audio_files_dir::{store_songs_directory,  get_songs_directory};
use anyhow::Result;

use crate::event_handler::SONGS;

use serde::{Serialize, Deserialize};

#[tauri::command]
pub fn set_song_dir(file_path: &str) -> Result<(), String> {
    println!("file path: {}", file_path);

    let file_path = file_path.replace("\\", "/");
    println!("file path: {}", file_path);
    store_songs_directory(&file_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command] 
pub fn get_songs_dir() -> Result<String, String> {
    let songs_dir = get_songs_directory().map_err(|e| e.to_string())?;
    match songs_dir {
        Some(song_dir) => Ok(song_dir),
        None => Err("song dir has not been set yet".to_string())
    }

}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

pub fn scan_dir(dir: &Path) -> Result<Vec<Song>> {
    let mut songs: Vec<Song> = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let sub_songs = scan_dir(&path)?;

                songs.extend(sub_songs);
            } else if let Some(extension) = path.extension() {
                if extension == "flac" || extension == "mp3" {
                    let song = Song {
                        path: path.to_string_lossy().to_string().replace("\\", "/"),
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        is_directory: false,
                    };
                    songs.push(song);
                }
            }
        }
    }

    Ok(songs)
}




#[tauri::command]
pub fn get_songs() -> Result<Vec<Song>, String> {

    let songs_guard = SONGS.lock().unwrap();
    let songs = songs_guard.clone();

    Ok(songs)
}
