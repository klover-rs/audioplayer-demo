import React from 'react';

import './ProgressBar.css';

interface AudioPlayerBarProps {
    value: number;
    min: number;
    max: number;
    onChange: (value: number) => void;
}

const ProgressBar: React.FC<AudioPlayerBarProps> = ({ value, min, max, onChange }) => {
    const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        onChange(Number(event.target.value));
    }

    return (
        <input
          type="range"
          className="progress-bar"
          value={value}
          min={min}
          max={max}
          onChange={handleChange}
        />
    );
}

export default ProgressBar;