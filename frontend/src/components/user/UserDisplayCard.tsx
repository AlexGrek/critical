import React from 'react';
import { Panel, List } from 'rsuite';
import { type UserWhoami } from '../user/AuthProvider'; // Adjust path as needed

interface UserDisplayCardProps {
    userData: UserWhoami;
}

const UserDisplayCard: React.FC<UserDisplayCardProps> = ({ userData }) => {
    return (
        <div className="flex justify-center p-4">
            <Panel
                header={<h3 className="text-2xl font-semibold">User Details</h3>}
                bordered
                className="w-full max-w-2xl"
            >
                <List hover>
                    <List.Item className="py-3 px-4 border-b border-gray-200 flex justify-between items-center">
                        <span className="font-medium">UID:</span>
                        <span className="text-right font-mono text-sm break-all">{userData.uid}</span>
                    </List.Item>
                    <List.Item className="py-3 px-4 border-b border-gray-200 flex justify-between items-center">
                        <span className="font-medium">Email:</span>
                        <span className="text-right text-blue-600 hover:underline">{userData.email}</span>
                    </List.Item>
                    <List.Item className="py-3 px-4 border-b border-gray-200 flex justify-between items-center">
                        <span className="font-medium">Role:</span>
                        <span className="text-right capitalize">{userData.role}</span>
                    </List.Item>
                    <List.Item className="py-3 px-4 border-b border-gray-200 flex justify-between items-center">
                        <span className="font-medium">Created At:</span>
                        <span className="text-right text-sm">{new Date(userData.created_at).toLocaleString()}</span>
                    </List.Item>
                    <List.Item className="py-3 px-4 border-b border-gray-200 flex justify-between items-center">
                        <span className="font-medium">Updated At:</span>
                        <span className="text-right text-sm">{new Date(userData.updated_at).toLocaleString()}</span>
                    </List.Item>
                    <List.Item className="py-3 px-4 flex justify-between items-start">
                        <span className="font-medium">Hashed Password:</span>
                        <span className="text-right font-mono text-xs break-all ml-4 max-w-[60%] select-all">
                            {userData.hashed_password}
                        </span>
                    </List.Item>
                    {Object.keys(userData.metadata).length > 0 && (
                        <List.Item className="py-3 px-4 border-t border-gray-200">
                            <div className="font-medium mb-2">Metadata:</div>
                            <ul className="">
                                {Object.entries(userData.metadata).map(([key, value]) => (
                                    <li key={key} className="break-all">
                                        <span className="font-semibold">{key}:</span> {JSON.stringify(value)}
                                    </li>
                                ))}
                            </ul>
                        </List.Item>
                    )}
                </List>
            </Panel>
        </div>
    );
};

export default UserDisplayCard;