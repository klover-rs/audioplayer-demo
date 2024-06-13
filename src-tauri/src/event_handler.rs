

use tauri::Manager;
use creek::{ReadDiskStream, ReadStreamOptions, SymphoniaDecoder};

use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use tokio::time::sleep as tsleep;
use std::time::Duration;
use std::sync::{mpsc, Mutex};
use tokio::sync::mpsc as tmpsc;

use crate::song_dir::scan_dir;

use crate::audio_controls::{handle_pause, handle_play, handle_repeat, handle_restart, handle_seek};
use crate::audio_backend::output;
use crate::audio_backend::get_all_audio_devices::get_device_info;
use crate::util::lmdb::audio_files_dir::get_songs_directory;

use crate::song_dir::Song;

const PLAY: &str = "play";
const PAUSE: &str = "pause";
const RESTART: &str = "restart";
//const LOOPING_POS: &str = "looping-pos";
const SEEK: &str = "seek";
const REPEAT: &str = "repeat";
const SWITCH_TRACK: &str = "switch_track";
const SKIP_TO_NEXT: &str = "skip_to_next";
const SKIP_TO_PREV: &str = "skip_to_prev";

pub enum GuiToProcessMsg {
    UseStream((Box<ReadDiskStream<SymphoniaDecoder>>, usize)),
    SetLoop { start: usize, end: usize },
    PlayResume,
    Pause,
    Repeat(bool),
    Restart,
    SeekTo(usize)
}

pub enum ProcessToGuiMsg {
    PlaybackPos(usize),
    Buffering(bool),
    TotalFrames(usize),
    DropOldStream(Box<ReadDiskStream<SymphoniaDecoder>>),
    DropAndNext(usize),
}

pub enum ControlMessage {
    Play,
    Pause,
    Restart,
    Repeat(bool),
    Seek(usize),
    SetTrack(usize),
    SkipToNext,
    SkipToPrev,
}

lazy_static::lazy_static! {
    pub static ref SONGS: Mutex<Vec<Song>> = Mutex::new(Vec::new());
    pub static ref CURRENT_TRACK_INDEX: Mutex<Option<(usize, usize)>> = Mutex::new(None);
}

fn start_playing_thread(
    from_gui_rx: Receiver<GuiToProcessMsg>,
    to_gui_tx: Sender<ProcessToGuiMsg>,
    to_process_tx: Sender<GuiToProcessMsg>,
    rx: mpsc::Receiver<ControlMessage>,
) {

    let to_gui_tx = to_gui_tx;

    std::thread::spawn(move || {
        let to_process_tx_clone = to_process_tx.clone();

        let (switch_track_tx, switch_track_rx) = mpsc::channel();
        
        let _cpal_stream: cpal::Stream = output::spawn_cpal_stream(to_gui_tx, from_gui_rx);

        std::thread::spawn(move || {

            let to_process_tx_clone = to_process_tx_clone.clone();

            
            println!("does the thread start?");
         
            while let Ok(track_index) = switch_track_rx.recv() {
                
                let songs_vec = SONGS.lock().unwrap();

                let songs_len = songs_vec.len();
                if track_index <= songs_len {

                    let opts: ReadStreamOptions<SymphoniaDecoder> = ReadStreamOptions {
                        num_cache_blocks: 20,
                        num_caches: 2,
                        ..Default::default()
                    };

                    let song: &Song = &songs_vec[track_index];

                    
                    let mut current_track_index = CURRENT_TRACK_INDEX.lock().unwrap();

                    *current_track_index = Some((track_index, songs_len));

                    let mut read_stream = match ReadDiskStream::<SymphoniaDecoder>::new(&song.path, 0, opts) {
                        Ok(stream) => stream,
                        Err(e) => {
                            eprintln!("error: {}", e);
                            continue;
                        }
                    };

                    let _ = read_stream.cache(0, 0);
                    read_stream.seek(0, Default::default()).unwrap();
                    read_stream.block_until_ready().unwrap();
         
                    let num_frames = read_stream.info().num_frames;
        
                    to_process_tx_clone.send(GuiToProcessMsg::UseStream((Box::new(read_stream), track_index))).unwrap();
                    to_process_tx_clone.send(GuiToProcessMsg::SetLoop { start: 0, end: num_frames }).unwrap();

                    to_process_tx_clone.send(GuiToProcessMsg::PlayResume).unwrap();
                }


            }

            std::thread::sleep(Duration::from_millis(2));
            
        });

        std::thread::spawn(move || {            
            while let Ok(msg) = rx.recv() {
                match msg {
                    ControlMessage::Play => {
                        handle_play(&mut to_process_tx.clone());
                    }
                    ControlMessage::Pause => {
                        handle_pause(&mut to_process_tx.clone());
                    }
                    ControlMessage::Restart => {
                        handle_restart(&mut to_process_tx.clone());
                    }
                    ControlMessage::Repeat(state) => {
                        handle_repeat(&mut to_process_tx.clone(), state);
                    }
                    ControlMessage::Seek(seek_to) => {
                        handle_seek(&mut to_process_tx.clone(), seek_to);
                    }
                    ControlMessage::SetTrack(track_index) => {
                        switch_track_tx.send(track_index).unwrap();
                    }
                    ControlMessage::SkipToNext => {
                        let track_index_guard = CURRENT_TRACK_INDEX.lock().unwrap();
                        if let Some((current_track_index, song_len)) = *track_index_guard {
                            if current_track_index < song_len {
                                switch_track_tx.send(current_track_index + 1).unwrap();
                            } else {
                                switch_track_tx.send(0).unwrap();
                            }
                        }
                    }           
                    ControlMessage::SkipToPrev => {
                        let track_index_guard = CURRENT_TRACK_INDEX.lock().unwrap();
                        if let Some((current_track_index, _song_len)) = *track_index_guard {
                            if current_track_index > 0 {
                                switch_track_tx.send(current_track_index - 1).unwrap();
                            } else {
                                switch_track_tx.send(0).unwrap();
                            }
                        }
                    }       
                    
                }

                thread::sleep(Duration::from_micros(10000));
            }
    
        });

        loop {
            std::thread::sleep(Duration::from_secs(5));
        }
    

    });
}


pub async fn event_handler(app_handle: tauri::AppHandle) {

    let (to_gui_tx, from_process_rx) = mpsc::channel();
    let (to_process_tx, from_gui_rx) = mpsc::channel();

    let (tx, rx) = mpsc::channel();

    tokio::task::spawn(async move {
        
        let mut files: Vec<Song> = Vec::new();

        loop {
            match get_songs_directory() {
                Ok(songs_dir) => {
                    match songs_dir {
                        Some(song_dir) => {
                            let song_dir_p = Path::new(&song_dir);
    
                            files = scan_dir(song_dir_p).unwrap();
    
                            if !files.is_empty() {
                                let mut songs_vec: std::sync::MutexGuard<Vec<Song>> = SONGS.lock().unwrap();
                                *songs_vec = files;
                                drop(songs_vec);
                            }
                            
                        }
                        None => {
                            println!("none");
                        }
                    }
                },
                Err(e) => {
                    println!("error: {:?}", e);
                }
            };
    
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    start_playing_thread(from_gui_rx, to_gui_tx, to_process_tx, rx);
    
    app_handle.listen_global(PLAY, {
        let tx = tx.clone();
        move |_event| {
            tx.send(ControlMessage::Play).unwrap();
        }
    });
    app_handle.listen_global(PAUSE, {
        let tx = tx.clone();
        move |_event| {
            tx.send(ControlMessage::Pause).unwrap();
        }
    });
    app_handle.listen_global(RESTART,  {
        let tx = tx.clone();
        move |_event| {
            tx.send(ControlMessage::Restart).unwrap();
        }
    });
    app_handle.listen_global(REPEAT,  {
        let tx = tx.clone();
        move |event| {

            let event_payload = event.payload().unwrap();

            let json_event: serde_json::Value = serde_json::from_str(event_payload).unwrap();

            let state = json_event.get("state").unwrap().as_bool().unwrap();

            tx.send(ControlMessage::Repeat(state)).unwrap();
        }
    });
    app_handle.listen_global(SEEK, {
        let tx = tx.clone();
        move |event| {
            let event_payload = event.payload().unwrap();

            let json_event: serde_json::Value = serde_json::from_str(event_payload).unwrap();

            let pos = json_event.get("pos").unwrap().as_u64().unwrap();
            
            tx.send(ControlMessage::Seek(pos as usize)).unwrap();
            println!("event : {:?}", pos);
        }
    });
    app_handle.listen_global(SWITCH_TRACK, {
        let tx = tx.clone();
        move |event| {
            let event_payload = event.payload().unwrap();

            let json_event: serde_json::Value = serde_json::from_str(event_payload).unwrap();

            let track_index = json_event.get("track_index").unwrap().as_u64().unwrap();

            tx.send(ControlMessage::SetTrack(track_index as usize)).unwrap();
        }
    });
    app_handle.listen_global(SKIP_TO_NEXT, {
        let tx = tx.clone();
        move |_event| {
            println!("do we receive the msg");
            tx.send(ControlMessage::SkipToNext).unwrap();
        }
    });
    app_handle.listen_global(SKIP_TO_PREV, {
        let tx = tx.clone();
        move |_event| {
            tx.send(ControlMessage::SkipToPrev).unwrap();
        }
    });


    //yea intentions behind these variable names: t is just a short for tokio xd | yea and same for tmpsc as you can see

    let (ttx, mut trx) = tmpsc::channel(100);

    std::thread::spawn(move || {
        loop {
            while let Ok(msg) = from_process_rx.try_recv() {
                match msg {
                    ProcessToGuiMsg::PlaybackPos(pos) => {
                        ttx.try_send(ProcessToGuiMsg::PlaybackPos(pos)).unwrap();
                    }
                    ProcessToGuiMsg::TotalFrames(frames) => {
                        ttx.try_send(ProcessToGuiMsg::TotalFrames(frames)).unwrap();
                    }
                    ProcessToGuiMsg::Buffering(buffering) => {
                        ttx.try_send(ProcessToGuiMsg::Buffering(buffering)).unwrap();
                    }
                    ProcessToGuiMsg::DropAndNext(last_index) => {
                        println!("drop and next song u slut. last index was: {}", last_index);
                        ttx.try_send(ProcessToGuiMsg::DropAndNext(last_index)).unwrap();
                    }
                    _ => {}
                }
            }
            //always remember to add some delay here, because else we will send too much events at once to the receiver which causes that the ui gets overwhelemed, and probably several other issues.
            // test for yourself which delay works for your system best here 
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    {
        let app_handle = app_handle.clone();

        tokio::spawn(async move {
            loop {
                while let Some(msg) = trx.recv().await {
                    match msg {
                        ProcessToGuiMsg::PlaybackPos(pos) => {
                            app_handle.emit_all("pos-frames", pos).unwrap();
                        }
                        ProcessToGuiMsg::TotalFrames(frames) => {
                            app_handle.emit_all("total-frames", frames).unwrap()
                        }
                        ProcessToGuiMsg::Buffering(status) => {
                            app_handle.emit_all("buffer-status", status).unwrap();
                        }
                        ProcessToGuiMsg::DropAndNext(last_index) => {
                            app_handle.emit_all("drop-and-next", last_index).unwrap();
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(1)).await;

                
            }
    
        });
    }

    {

        let app_handle = app_handle.clone();

        tokio::task::spawn(async move {

            loop {
                let audio_devices = get_device_info().unwrap();
    
                app_handle.emit_all("get-all-devices", audio_devices).unwrap();
    
                tsleep(Duration::from_secs(1)).await;
            }
    
           
        });
    }

}

