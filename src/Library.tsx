import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/tauri"
import Player from "./Player";
import './Library.css';
import { emit } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";

interface SongDir {
    path: string;
    name: string;
}

export default function Library() {

    const [songDirPath, setSongDirPath] = useState("");
    const [songsDir, setSongsDir] = useState<SongDir[]>([]);

    const nav = useNavigate();

    useEffect(() => {
        const getSongDirPath = async () => {
            try {
                let result: string = await invoke("get_songs_dir");
                setSongDirPath(result);
                
            } catch (e) {
                console.error(e)
            }
        }

        const getSongs = async () => {
            try {
                let result: SongDir[] = await invoke("get_songs");
                setSongsDir(result);

            } catch (e) {
                console.error(e);
            }
        }

        getSongDirPath();
        getSongs();
    }, []);

    const switchTrack = (track_index: number) => {
        emit("switch_track", {
            track_index: track_index
        });
    }

 

    return (
        <div className="main-content">
            
            <div className="library-container">
            <div className="library-header">
                <div className="library-header-inner">
                    <button className="go-back-btn" onClick={() => nav("/")}>go back</button>
                    <p>{songDirPath}</p>
                </div>
            </div>
            <div className="library-content">
                <ul>
                    {songsDir.map((song, index) => (
                        <div className="library-items">
                            <div className="library-items-inner">
                                <li className="library-items-li" key={index}>
                                    <p className="song-name">{song.name}</p>
                                    <button onClick={() => switchTrack(index)}>play! :3</button>
                                    <p className="song-path">{song.path}</p>
                                </li> 
                            </div>
                        </div>
                    ))}
                </ul>
            </div>
            </div>
            {/* well calculate here -1, else it will crash for some reaosn bleh*/}
            <Player/>
        </div>
    );
}