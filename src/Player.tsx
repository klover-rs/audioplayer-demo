
import { useEffect, useState } from "react";
import "./Player.css";
import ProgressBar from "./ui_components/ProgressBar";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

interface EventConstants {
    EVENT_PLAY: string;
    EVENT_PAUSE: string;
    EVENT_RESTART: string;
    EVENT_REPEAT: string;
    EVENT_SEEK: string;
    EVENT_SKIP_NEXT: string;
    EVENT_SKIP_PREV: string;
}
  
const eventConstants: EventConstants = {
    EVENT_PLAY: 'play',
    EVENT_PAUSE: 'pause',
    EVENT_RESTART: 'restart',
    EVENT_REPEAT: 'repeat',
    EVENT_SEEK: 'seek',
    EVENT_SKIP_NEXT: 'skip_to_next',
    EVENT_SKIP_PREV: 'skip_to_prev'
};

enum Skip {
  SkipToPrev,
  SkipToNext
}

export default function Player() {
    
    const [isRepeating, setIsRepeating] = useState(false);
    const [currentFrames, setCurrentFrames] = useState(0);
    const [totalFrames, setTotalFrames] = useState(0);
    const [buffering, setBuffering] = useState(false);
  

    const switchTrack = (track_index: number) => {
      emit("switch_track", {
          track_index: track_index
      });
    }

   

    const totalsongs = async () => {
      try {
        const result: number[] = await invoke("get_current_index");
        return result[1];
      } catch(e) {
        console.error(e);
      }
    }
    

    useEffect(() => {
        let isSubscribed = true;

        const fetchCurrentFrames = async () => {
          try {
            const unlisten = await listen('pos-frames', (event: any) => {
              if (isSubscribed) {
                setCurrentFrames(event.payload);
              }
            });
            return unlisten;
          } catch (e) {
            console.error(e);
          }
    
        }
    
        const fetchTotalFrames = async () => {
          try {
            const unlisten = await listen('total-frames', (event: any) => {
              if (isSubscribed) {
                setTotalFrames(event.payload);
              }
            });
            return unlisten;
          } catch (e) {
            console.error(e);
          }
        }
    
        const fetchBufferStatus = async () => {
          try {
            const unlisten = await listen('buffer-status', (event) => {
                
              if (typeof event.payload === 'boolean') {
                if (isSubscribed) {
                  setBuffering(event.payload);
                }
              }
            });
            return unlisten 
          } catch (e) {
            console.error(e);
          }
        } 



        const awaitDropAndNext = async () => {
            try {
              const unlisten = await listen("drop-and-next", async (event) => {
                let event_payload: number = Number(event.payload) + 1;
                let total_songs: number | undefined = await totalsongs();

                if (isSubscribed) {
                  console.log(event.payload);
                  if (total_songs !== undefined) {
                    if (event_payload <= total_songs) {
                      switchTrack(event_payload);
                    } else {
                      switchTrack(0);
                    }
                  }
                }
              });
              return unlisten 
            } catch(e) {
              console.error(e);
            }
        }

        const unlistenFunctions: any[] = [];

        fetchCurrentFrames().then(unlisten => unlistenFunctions.push(unlisten));
        fetchTotalFrames().then(unlisten => unlistenFunctions.push(unlisten));
        fetchBufferStatus().then(unlisten => unlistenFunctions.push(unlisten));
        awaitDropAndNext().then(unlisten => unlistenFunctions.push(unlisten));
       

        return () => {
          isSubscribed = false;

          unlistenFunctions.forEach(unlisten => unlisten && unlisten());
        }

      }, []);


    

    const send_event = async (event: string) => {
        switch (event) {
          case eventConstants.EVENT_PLAY:
            emit(eventConstants.EVENT_PLAY);
            break;
          case eventConstants.EVENT_PAUSE:
            emit(eventConstants.EVENT_PAUSE)
            break;
          case eventConstants.EVENT_RESTART:
            emit(eventConstants.EVENT_RESTART);
            break;
          case eventConstants.EVENT_REPEAT:
            emit(eventConstants.EVENT_REPEAT, {
              state: isRepeating
            });
            console.log("repeat state: " + isRepeating);
            break;
          default:
            break;
        }
      }
    

    const handleSeek = (pos: number) => {
        emit(eventConstants.EVENT_SEEK, {
          pos: pos
        });
    }

    const handleSkip = async (skip: Skip) => {
      try {
        
        switch (skip) {
          case Skip.SkipToNext:
            emit(eventConstants.EVENT_SKIP_NEXT);
            break;
          case Skip.SkipToPrev: 
            emit(eventConstants.EVENT_SKIP_PREV);
            break;
          default: 
            break;
        }
      } catch (e) {
        console.error(e);
      }
    } 

    return (
        <div className="player">
            <ProgressBar
                value={currentFrames}
                min={0}
                max={totalFrames}
                onChange={(value) => {
                    handleSeek(value);
                }}
            />
            <br/>
            <div className="control-btns">
                <div className="control-btns-inner">
                <button className="control-btn" onClick={() => handleSkip(Skip.SkipToPrev)}>prev track</button>
                <button className="control-btn" onClick={() => send_event(eventConstants.EVENT_PLAY)}>play music!</button>
                <button className="control-btn" onClick={() => send_event(eventConstants.EVENT_PAUSE)}>pause music!</button>
                <button className="control-btn" onClick={() => send_event(eventConstants.EVENT_RESTART)}>restart track!</button>
                <button className="control-btn" onClick={() => {
                    setIsRepeating(prevValue => !prevValue);
                    send_event(eventConstants.EVENT_REPEAT)
                }}>repeat button yay!</button>
                <button className="control-btn" onClick={() => handleSkip(Skip.SkipToNext)}>next track</button>
                </div>
                
            </div>
            <div className="player-state">
                <div className="player-state-inner">
                    <h4 className="frames-count">{currentFrames} / {totalFrames}</h4>
                    {buffering ? <h4 className="is-buffering">buffering</h4> : <h4 className="is-buffered">buffered!</h4>}
                </div>
            </div>
            
        </div>
    )
}