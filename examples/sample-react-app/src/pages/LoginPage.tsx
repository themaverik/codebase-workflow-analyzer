import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import axios from 'axios';

interface LoginFormData {
  email: string;
  password: string;
  rememberMe: boolean;
}

interface LoginPageProps {
  onLogin?: (token: string) => void;
}

const LoginPage: React.FC<LoginPageProps> = ({ onLogin }) => {
  const [formData, setFormData] = useState<LoginFormData>({
    email: '',
    password: '',
    rememberMe: false,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const response = await axios.post('/api/auth/login', {
        email: formData.email,
        password: formData.password,
        rememberMe: formData.rememberMe,
      });

      const { token } = response.data;
      
      // Store token
      localStorage.setItem('authToken', token);
      
      // Call onLogin callback if provided
      if (onLogin) {
        onLogin(token);
      }

      // Navigate to dashboard
      navigate('/');
    } catch (err) {
      if (axios.isAxiosError(err) && err.response?.status === 401) {
        setError('Invalid email or password');
      } else {
        setError('Login failed. Please try again.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-container">
        <h1>Sign In</h1>
        
        <form onSubmit={handleSubmit} className="login-form">
          {error && (
            <div className="error-banner" role="alert">
              {error}
            </div>
          )}
          
          <div className="form-group">
            <label htmlFor="email">Email Address</label>
            <input
              id="email"
              name="email"
              type="email"
              required
              value={formData.email}
              onChange={handleInputChange}
              placeholder="Enter your email"
              disabled={loading}
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              name="password"
              type="password"
              required
              value={formData.password}
              onChange={handleInputChange}
              placeholder="Enter your password"
              disabled={loading}
            />
          </div>
          
          <div className="form-group checkbox-group">
            <input
              id="rememberMe"
              name="rememberMe"
              type="checkbox"
              checked={formData.rememberMe}
              onChange={handleInputChange}
              disabled={loading}
            />
            <label htmlFor="rememberMe">Remember me</label>
          </div>
          
          <button 
            type="submit" 
            className="login-button"
            disabled={loading}
          >
            {loading ? 'Signing In...' : 'Sign In'}
          </button>
        </form>
        
        <div className="login-footer">
          <a href="/forgot-password">Forgot your password?</a>
          <span>Don't have an account? <a href="/register">Sign up</a></span>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;