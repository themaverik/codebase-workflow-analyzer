import React, { useState, useEffect } from 'react';
import axios from 'axios';
import UserList from '../components/UserList';
import LoadingSpinner from '../components/LoadingSpinner';

interface DashboardData {
  users: User[];
  metrics: Metrics;
}

interface User {
  id: number;
  name: string;
  email: string;
  status: 'active' | 'inactive';
}

interface Metrics {
  totalUsers: number;
  activeUsers: number;
  newSignups: number;
}

const Dashboard: React.FC = () => {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        setLoading(true);
        const response = await axios.get('/api/dashboard');
        setData(response.data);
      } catch (err) {
        setError('Failed to load dashboard data');
      } finally {
        setLoading(false);
      }
    };

    fetchDashboardData();
  }, []);

  if (loading) {
    return <LoadingSpinner message="Loading dashboard..." />;
  }

  if (error) {
    return <div className="error-message">{error}</div>;
  }

  if (!data) {
    return <div>No data available</div>;
  }

  return (
    <div className="dashboard">
      <h1>Dashboard</h1>
      
      <div className="metrics-grid">
        <div className="metric-card">
          <h3>Total Users</h3>
          <span className="metric-value">{data.metrics.totalUsers}</span>
        </div>
        <div className="metric-card">
          <h3>Active Users</h3>
          <span className="metric-value">{data.metrics.activeUsers}</span>
        </div>
        <div className="metric-card">
          <h3>New Signups</h3>
          <span className="metric-value">{data.metrics.newSignups}</span>
        </div>
      </div>

      <div className="users-section">
        <h2>Recent Users</h2>
        <UserList users={data.users} onUserSelect={handleUserSelect} />
      </div>
    </div>
  );

  function handleUserSelect(user: User) {
    // TODO: Implement user selection logic
    console.log('Selected user:', user);
  }
};

export default Dashboard;