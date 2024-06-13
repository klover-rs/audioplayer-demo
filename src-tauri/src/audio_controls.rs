use std::sync::mpsc::Sender;
use crate::event_handler::GuiToProcessMsg;
use crate::event_handler::CURRENT_TRACK_INDEX;

pub fn handle_play(to_player_tx: &mut Sender<GuiToProcessMsg>)  {

    to_player_tx.send(GuiToProcessMsg::PlayResume).unwrap();
    println!("send playback resume signal")
}

pub fn handle_pause(to_player_tx: &mut Sender<GuiToProcessMsg>) {
    to_player_tx.send(GuiToProcessMsg::Pause).unwrap();
}

pub fn handle_restart(to_player_tx: &mut Sender<GuiToProcessMsg>)  {

    to_player_tx.send(GuiToProcessMsg::Restart).unwrap();
    println!("send playback resume signal")
}

/*pub fn handle_looping_pos(loop_start: usize, loop_end: usize) {
    
}*/

pub fn handle_seek(to_player_tx: &mut Sender<GuiToProcessMsg>, frame: usize) {
    to_player_tx.send(GuiToProcessMsg::SeekTo(frame)).unwrap();

}

pub fn handle_repeat(to_player_tx: &mut Sender<GuiToProcessMsg>, state: bool) {
    to_player_tx.send(GuiToProcessMsg::Repeat(state)).unwrap();
}

#[tauri::command]
pub fn get_current_index() -> Result<Option<(usize, usize)>, String> {
    let index_guard = CURRENT_TRACK_INDEX.lock().unwrap();

    let current_index = *index_guard;

    Ok(current_index)
}