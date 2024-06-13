import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
import App from './App';
import Library from './Library';
import './style.css';

export default function DomRouter() {
    return (
        <Router>
            <Routes>
                <Route path='/' element={<App />} />
                <Route path='/library' element={<Library />} />
            </Routes>
        </Router>
    )
}