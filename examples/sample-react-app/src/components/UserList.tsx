import React, { useState } from 'react';

interface User {
  id: number;
  name: string;
  email: string;
  status: 'active' | 'inactive';
}

interface UserListProps {
  users: User[];
  onUserSelect?: (user: User) => void;
  maxDisplayCount?: number;
}

const UserList: React.FC<UserListProps> = ({ 
  users, 
  onUserSelect, 
  maxDisplayCount = 10 
}) => {
  const [filter, setFilter] = useState<'all' | 'active' | 'inactive'>('all');

  const filteredUsers = users.filter(user => {
    if (filter === 'all') return true;
    return user.status === filter;
  }).slice(0, maxDisplayCount);

  const handleUserClick = (user: User) => {
    if (onUserSelect) {
      onUserSelect(user);
    }
  };

  if (users.length === 0) {
    return <div className="empty-state">No users found</div>;
  }

  return (
    <div className="user-list">
      <div className="user-list-filters">
        <button 
          className={filter === 'all' ? 'active' : ''}
          onClick={() => setFilter('all')}
        >
          All ({users.length})
        </button>
        <button 
          className={filter === 'active' ? 'active' : ''}
          onClick={() => setFilter('active')}
        >
          Active ({users.filter(u => u.status === 'active').length})
        </button>
        <button 
          className={filter === 'inactive' ? 'active' : ''}
          onClick={() => setFilter('inactive')}
        >
          Inactive ({users.filter(u => u.status === 'inactive').length})
        </button>
      </div>

      <div className="user-list-items">
        {filteredUsers.map(user => (
          <div 
            key={user.id}
            className={`user-item ${user.status}`}
            onClick={() => handleUserClick(user)}
          >
            <div className="user-avatar">
              {user.name.charAt(0).toUpperCase()}
            </div>
            <div className="user-details">
              <h4>{user.name}</h4>
              <p>{user.email}</p>
            </div>
            <div className={`user-status ${user.status}`}>
              {user.status}
            </div>
          </div>
        ))}
      </div>

      {filteredUsers.length < users.length && (
        <div className="user-list-footer">
          Showing {filteredUsers.length} of {users.length} users
        </div>
      )}
    </div>
  );
};

export default UserList;