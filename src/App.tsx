import { useEffect, useState } from "react";
import { listen } from '@tauri-apps/api/event'
import "./App.css";


import FilePickerComponent from "./ui_components/FilePicker";
import Player from "./Player";
import { useNavigate } from "react-router-dom";



function App() {

  //const [seekPos, setSeekPos] = useState(0);

  const [availableDevices, setAvailableDevices] = useState<string[]>([]);

  const nav = useNavigate();

  useEffect(() => {
    const fetchAudioDevices = async () => {
      try {
        await listen('get-all-devices', (event: any) => {
          if (Array.isArray(event.payload)) {
            setAvailableDevices(event.payload);
          }
        });


      } catch (e) {
        console.log(e);
      }

    } 
    fetchAudioDevices();
  }, [])

  return (
    <div className="container">
      <FilePickerComponent />
      <br/>
      <div className="all-devices">
        <h3>all of your audio devices :D</h3>
        {availableDevices.length > 0 ? (
          <ul>
            {availableDevices.map((device, index) => (
              <li key={index}>{device}</li>
            ))}
          </ul>
        ) : (
          <p>no devices :(</p>
        )}
      </div>
      <br/>
      <div className="go-to-lib">
        <div className="go-to-lib-inner">
          <button className="go-to-library-btn" onClick={() => nav("/library")}>Go To Library</button>
        </div>
      </div>
      <Player/>
    </div>
  );
}



export default App;
