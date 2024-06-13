# audioplayer-demo
this is a demo for a next upcoming planned project, THIS IS A DEMO, YOU WILL ENCOUNTER ISSUES. A audio player written in rust using cpal and creek in combination with reactjs and tauri

## How to build

### Prerequisites
- [Node.js](https://nodejs.org/en/download/package-manager)
- [Rust](https://www.rust-lang.org/tools/install)

### Building the project
navigate into the source directory and run the following commands in your terminal
- `npm i` 
- `npm run tauri dev` for debugging, or `npm run tauri build` for a build of the application

please plan in that this project will create a folder in your home dir called "lmdb_data" this is just used to store configurations

### important!! 
in case you encounter issues while building, like i experienced with my arm macbook, please make sure to verify, that it is really my fault by creating a new tauri project and see if that works

## How to use 
1. click on "choose directory" and select the directory containing your audio files (atm: .mp3 and .flac are supported)
![image](https://github.com/mari-rs/audioplayer-demo/assets/98649425/a8006b69-2535-46a5-8995-dd18ebfa7b8e)
2. after you have done that, click on the button which says "Go To Library"
3. there you will see all of the playable songs in the directory
![image](https://github.com/mari-rs/audioplayer-demo/assets/98649425/40b8f2ae-f4c1-46b1-8167-90beee9cf7fa)
4. i am sure you can figure the rest yourself out :3

in case you encounter issues in debug mode like higher buffering times, stutters in audio after seeking etc. This is normal, in release mode you wont experience these issues. 
