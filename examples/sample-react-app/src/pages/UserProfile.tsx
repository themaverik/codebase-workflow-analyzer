import React, { useState, useEffect } from 'react';
import axios from 'axios';

interface UserData {
  id: number;
  name: string;
  email: string;
  avatar?: string;
  bio?: string;
  location?: string;
  website?: string;
  joinedDate: string;
}

const UserProfile: React.FC = () => {
  const [userData, setUserData] = useState<UserData | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // TODO: Replace with actual user ID from auth context
    const userId = 1;
    
    const fetchUserData = async () => {
      try {
        const response = await axios.get(`/api/users/${userId}`);
        setUserData(response.data);
      } catch (error) {
        console.error('Failed to fetch user data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchUserData();
  }, []);

  const handleSave = async (updatedData: Partial<UserData>) => {
    if (!userData) return;

    try {
      const response = await axios.put(`/api/users/${userData.id}`, updatedData);
      setUserData(response.data);
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to update profile:', error);
    }
  };

  if (loading) {
    return <div className="loading">Loading profile...</div>;
  }

  if (!userData) {
    return <div className="error">Failed to load user profile</div>;
  }

  return (
    <div className="user-profile">
      <div className="profile-header">
        <div className="avatar-section">
          {userData.avatar ? (
            <img src={userData.avatar} alt="Profile" className="avatar" />
          ) : (
            <div className="avatar-placeholder">
              {userData.name.charAt(0).toUpperCase()}
            </div>
          )}
        </div>
        
        <div className="profile-info">
          <h1>{userData.name}</h1>
          <p className="email">{userData.email}</p>
          <p className="join-date">
            Joined {new Date(userData.joinedDate).toLocaleDateString()}
          </p>
        </div>
        
        <div className="profile-actions">
          <button 
            onClick={() => setIsEditing(!isEditing)}
            className="edit-button"
          >
            {isEditing ? 'Cancel' : 'Edit Profile'}
          </button>
        </div>
      </div>

      <div className="profile-content">
        {isEditing ? (
          <EditProfileForm 
            userData={userData} 
            onSave={handleSave}
            onCancel={() => setIsEditing(false)}
          />
        ) : (
          <ProfileDisplay userData={userData} />
        )}
      </div>
    </div>
  );
};

// TODO: Extract to separate component file
const ProfileDisplay: React.FC<{ userData: UserData }> = ({ userData }) => (
  <div className="profile-display">
    <section>
      <h3>About</h3>
      <p>{userData.bio || 'No bio available'}</p>
    </section>
    
    <section>
      <h3>Details</h3>
      <div className="details-grid">
        <div>
          <strong>Location:</strong>
          <span>{userData.location || 'Not specified'}</span>
        </div>
        <div>
          <strong>Website:</strong>
          <span>
            {userData.website ? (
              <a href={userData.website} target="_blank" rel="noopener noreferrer">
                {userData.website}
              </a>
            ) : (
              'Not specified'
            )}
          </span>
        </div>
      </div>
    </section>
  </div>
);

// TODO: Implement edit form component
const EditProfileForm: React.FC<{
  userData: UserData;
  onSave: (data: Partial<UserData>) => void;
  onCancel: () => void;
}> = ({ userData, onSave, onCancel }) => {
  const [formData, setFormData] = useState({
    name: userData.name,
    bio: userData.bio || '',
    location: userData.location || '',
    website: userData.website || '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(formData);
  };

  return (
    <form onSubmit={handleSubmit} className="edit-profile-form">
      <div className="form-group">
        <label htmlFor="name">Name</label>
        <input
          id="name"
          type="text"
          value={formData.name}
          onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="bio">Bio</label>
        <textarea
          id="bio"
          value={formData.bio}
          onChange={(e) => setFormData(prev => ({ ...prev, bio: e.target.value }))}
          rows={4}
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="location">Location</label>
        <input
          id="location"
          type="text"
          value={formData.location}
          onChange={(e) => setFormData(prev => ({ ...prev, location: e.target.value }))}
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="website">Website</label>
        <input
          id="website"
          type="url"
          value={formData.website}
          onChange={(e) => setFormData(prev => ({ ...prev, website: e.target.value }))}
        />
      </div>
      
      <div className="form-actions">
        <button type="submit" className="save-button">Save Changes</button>
        <button type="button" onClick={onCancel} className="cancel-button">Cancel</button>
      </div>
    </form>
  );
};

export default UserProfile;