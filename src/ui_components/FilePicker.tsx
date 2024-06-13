import { dialog, invoke } from '@tauri-apps/api';
import React from 'react';

const FilePickerComponent: React.FC = () => {
    
    const handleFileSelection = async () => {
        const result = await dialog.open({
            directory: true,
            multiple: false
        }).catch(console.error);
        await invoke('set_song_dir', {
            filePath: result
        })
    }

    return (
        <div>
            <button onClick={handleFileSelection}>Choose Directory</button>
        </div>
    )
}

export default FilePickerComponent;