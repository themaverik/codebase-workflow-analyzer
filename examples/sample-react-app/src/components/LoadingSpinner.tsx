import React from 'react';

interface LoadingSpinnerProps {
  message?: string;
  size?: 'small' | 'medium' | 'large';
}

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ 
  message = 'Loading...', 
  size = 'medium' 
}) => {
  return (
    <div className={`loading-spinner ${size}`}>
      <div className="spinner" />
      {message && <p className="loading-message">{message}</p>}
    </div>
  );
};

export default LoadingSpinner;