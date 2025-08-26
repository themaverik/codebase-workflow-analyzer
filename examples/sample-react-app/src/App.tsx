import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import LoginPage from './pages/LoginPage';
import UserProfile from './pages/UserProfile';
import NavBar from './components/NavBar';
import './App.css';

interface AppProps {
  theme?: 'light' | 'dark';
}

const App: React.FC<AppProps> = ({ theme = 'light' }) => {
  return (
    <Router>
      <div className={`app app-${theme}`}>
        <NavBar />
        <main className="app-content">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/login" element={<LoginPage />} />
            <Route path="/profile" element={<UserProfile />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
};

export default App;