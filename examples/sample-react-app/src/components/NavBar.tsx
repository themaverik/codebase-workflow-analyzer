import React from 'react';
import { Link, useLocation } from 'react-router-dom';

interface NavBarProps {
  logo?: string;
  onLogout?: () => void;
}

const NavBar: React.FC<NavBarProps> = ({ logo, onLogout }) => {
  const location = useLocation();

  const isActiveLink = (path: string): boolean => {
    return location.pathname === path;
  };

  return (
    <nav className="navbar">
      <div className="navbar-brand">
        {logo ? (
          <img src={logo} alt="Logo" className="logo" />
        ) : (
          <span className="brand-text">Sample App</span>
        )}
      </div>
      
      <div className="navbar-nav">
        <Link 
          to="/" 
          className={`nav-link ${isActiveLink('/') ? 'active' : ''}`}
        >
          Dashboard
        </Link>
        <Link 
          to="/profile" 
          className={`nav-link ${isActiveLink('/profile') ? 'active' : ''}`}
        >
          Profile
        </Link>
      </div>
      
      <div className="navbar-actions">
        <button 
          onClick={onLogout}
          className="logout-button"
        >
          Logout
        </button>
      </div>
    </nav>
  );
};

export default NavBar;