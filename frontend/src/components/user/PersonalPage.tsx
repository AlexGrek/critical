// src/App.tsx
import React, { useEffect, useState } from 'react';
import UserDisplayCard from './UserDisplayCard';
import { useAuth, type UserWhoami } from '../user/AuthProvider'; // Adjust path as needed

const PersonalPage: React.FC = () => {
    const auth = useAuth();

    const [user, setUser] = useState<UserWhoami | null>(null);

    useEffect(() => {
        setUser(auth.getUserInfo())
    }, [auth])

    // Example with metadata
    const userDataWithMetadata: UserWhoami = {
        uid: 'METADATA_USER',
        email: 'meta@example.com',
        hashed_password: 'some_other_hash',
        role: 'admin',
        metadata: {
            organization: 'Acme Corp',
            permissions: ['read', 'write', 'delete'],
            last_login_ip: '192.168.1.1',
        },
        created_at: '2025-06-13T10:00:00.000Z',
        updated_at: '2025-06-14T11:30:00.000Z',
    };


    return (
        <div>
            {user != null && <UserDisplayCard userData={user} />}
            <UserDisplayCard userData={userDataWithMetadata} />
        </div>
    );
};

export default PersonalPage;