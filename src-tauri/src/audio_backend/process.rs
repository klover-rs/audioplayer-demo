


use creek::read::ReadError;
use creek::{Decoder, ReadDiskStream, SeekMode, SymphoniaDecoder};

use std::sync::mpsc::{Receiver, Sender};

use crate::event_handler::{GuiToProcessMsg, ProcessToGuiMsg};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Paused, 
    Playing
}

pub struct Process {
    read_disk_stream: Option<Box<ReadDiskStream<SymphoniaDecoder>>>,

    to_gui_tx: Sender<ProcessToGuiMsg>,
    from_gui_rx: Receiver<GuiToProcessMsg>,
    

    playback_state: PlaybackState,
    repeat_state: bool,
    had_cache_miss_last_cycle: bool,

    loop_start: usize,
    loop_end: usize,

    index: usize,

    fatal_error: bool
}

impl Process {
    pub fn new(
        to_gui_tx: Sender<ProcessToGuiMsg>,
        from_gui_rx: Receiver<GuiToProcessMsg>,
    ) -> Self {
        Self {
            read_disk_stream: None,
            to_gui_tx,
            from_gui_rx,

            playback_state: PlaybackState::Paused,
            repeat_state: false,
            had_cache_miss_last_cycle: false,

            loop_start: 0,
            loop_end: 0,

            index: 0,

            fatal_error: false,
            
        }
    }

    pub fn process(&mut self, data: &mut [f32]) {
        if self.fatal_error {
            silence(data);
            return;
        }

        if let Err(e) = self.try_process(data) {
            if matches!(e, ReadError::FatalError(_)) {
                self.fatal_error = true;
            }

            println!("{:?}", e);
            silence(data);
        }

    }

    fn try_process(
        &mut self,
        mut data: &mut [f32],
    ) -> Result<(), ReadError<<SymphoniaDecoder as Decoder>::FatalError>> {
        // in the tauri version, maybe you shouldnt change how this works, but instead implement a middlewear which passes the calls from rtrb to tauri so it can return it to the front end and back

        while let Ok(msg) = self.from_gui_rx.try_recv() {
            match msg {
                GuiToProcessMsg::UseStream((read_disk_stream, index)) => {
                    self.playback_state = PlaybackState::Paused;
                    self.loop_start = 0;
                    self.loop_end = 0;

                    self.index = index;

                    if let Some(old_stream) = self.read_disk_stream.take() {
                        let _= self
                            .to_gui_tx
                            .send(ProcessToGuiMsg::DropOldStream(old_stream));
                    }

                    println!("set new stream!");

                    self.read_disk_stream = Some(read_disk_stream);
                }
                GuiToProcessMsg::SetLoop { start, end } => {
                    self.loop_start = start;
                    self.loop_end = end;

                    if start != 0 {
                        if let Some(read_disk_stream) = &mut self.read_disk_stream {
                            read_disk_stream.cache(1, start)?;
                        }
                    }
                }
                GuiToProcessMsg::PlayResume => {
                    self.playback_state = PlaybackState::Playing;
                }
                GuiToProcessMsg::Pause => {
                    self.playback_state = PlaybackState::Paused;
                }
                GuiToProcessMsg::Repeat(state) => {
                    self.repeat_state = state;
                }
                GuiToProcessMsg::Restart => {
                    self.playback_state = PlaybackState::Playing;

                    if let Some(read_disk_stream) = &mut self.read_disk_stream {
                        read_disk_stream.seek(self.loop_start, SeekMode::Auto)?;
                    }
                }
                GuiToProcessMsg::SeekTo(pos) => {
                    if let Some(read_disk_stream) = &mut self.read_disk_stream {
                        read_disk_stream.seek(pos, SeekMode::Auto)?;
                    }
                }
            }
        }

        let mut cache_missed_this_cycle = false; 
        let mut drop_stream = false;

        if let Some(read_disk_stream) = &mut self.read_disk_stream {
            if !read_disk_stream.is_ready()? {
                cache_missed_this_cycle = true;

                let _ = self.to_gui_tx.send(ProcessToGuiMsg::Buffering(true));
            } else {
                let _ = self.to_gui_tx.send(ProcessToGuiMsg::Buffering(false));
            }

            if let PlaybackState::Paused = self.playback_state {
                silence(data);
                return Ok(());
            }

            let num_frames = read_disk_stream.info().num_frames;

            let _ = self.to_gui_tx.send(ProcessToGuiMsg::TotalFrames(num_frames));

            let num_channels = usize::from(read_disk_stream.info().num_channels);

            while data.len() >= num_channels {
                let read_frames = data.len() / 2;

                let mut playhead = read_disk_stream.playhead();

                let loop_end =if playhead < self.loop_end {
                    self.loop_end
                } else {
                    num_frames
                };

                let read_data = read_disk_stream.read(read_frames)?;

                playhead += read_data.num_frames();
                if playhead >= loop_end {
                    let to_end_of_loop = read_data.num_frames() - (playhead - loop_end);

                    if read_data.num_channels() == 1 {
                        let ch = read_data.read_channel(0);

                        for i in 0..to_end_of_loop {
                            data[i * 2] = ch[i];
                            data[i * 2 + 1] = ch[i];
                        }
                    } else if read_data.num_channels() == 2{
                        let ch1 = read_data.read_channel(0);
                        let ch2 = read_data.read_channel(1);

                        for i in 0..to_end_of_loop {
                            data[i * 2] = ch1[i];
                            data[i * 2 + 1] = ch2[i];
                        }
                    }

                    read_disk_stream.seek(self.loop_start, SeekMode::Auto)?;

                    data = &mut data[to_end_of_loop * 2.. ];
                } else {
                    if read_data.num_channels() == 1 {
                        let ch = read_data.read_channel(0);

                        for i in 0..read_data.num_frames() {
                            data[i * 2] = ch[i];
                            data[i * 2 + 1] = ch[i];
                        }
                    } else if read_data.num_channels() == 2 {
                        let ch1 = read_data.read_channel(0);
                        let ch2 = read_data.read_channel(1);

                        for i in 0..read_data.num_frames() {
                            data[i * 2] = ch1[i];
                            data[i * 2 + 1] = ch2[i];
                        }
                    }

                    data = &mut data[read_data.num_frames() * 2..];
                }
            }

            if read_disk_stream.playhead() >= num_frames - 500 {
                println!("end of file reached.");
                if !self.repeat_state {
                    drop_stream = true;
                }
            }


            let _ =  self
                .to_gui_tx
                .send(ProcessToGuiMsg::PlaybackPos(read_disk_stream.playhead()));
        } else {
            silence(data);
        }

        

        if drop_stream {
            if let Some(dropped_stream) = self.read_disk_stream.take() {
                let _ = self.to_gui_tx.send(ProcessToGuiMsg::DropOldStream(dropped_stream));
                let _ = self.to_gui_tx.send(ProcessToGuiMsg::DropAndNext(self.index));
            }
        }

        if self.had_cache_miss_last_cycle {
            let buffer_size = data.len() as f32;
            for (i, sample) in data.iter_mut().enumerate() {
                *sample *= i as f32 / buffer_size;
            }
        }

        self.had_cache_miss_last_cycle = cache_missed_this_cycle;

        
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Some(stream) = self.read_disk_stream.take() {
            println!("the stream has finished playing, do something below: ");
            let _ = self.to_gui_tx.send(ProcessToGuiMsg::DropOldStream(stream));
        } else {
            println!("else was called in drop for some reason, what could that mean xd");
        }
    }
}

fn silence(data: &mut [f32]) {
    for sample in data.iter_mut() {
        *sample = 0.0;
    }
   
}