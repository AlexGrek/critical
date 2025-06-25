import React, { useState, useEffect, useCallback } from 'react';
import { Drawer, Loader, Placeholder, Message, Button, Divider } from 'rsuite';
import { useMediaQuery } from 'react-responsive'; // Import useMediaQuery from react-responsive

// Define the shape of a notification
interface Notification {
  id: string;
  title: string;
  message: string;
  timestamp: string; // Or a Date object if you prefer
}

// Define the props for the NotificationsDrawer component
interface NotificationsDrawerProps {
  open: boolean;
  onClose: () => void;
}

const NotificationsDrawer: React.FC<NotificationsDrawerProps> = ({ open, onClose }) => {
  // Use react-responsive's useMediaQuery hook
  const isMobile = useMediaQuery({ maxWidth: 768 }); // Adjust breakpoint as needed

  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Simulate an API call to fetch notifications
  const fetchNotifications = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Simulate a successful API response
      const mockNotifications: Notification[] = [
        { id: '1', title: 'New Message', message: 'You have a new message from John Doe.', timestamp: new Date().toLocaleString() },
        { id: '2', title: 'Update Available', message: 'Version 2.1 is now available for download.', timestamp: new Date(Date.now() - 3600000).toLocaleString() }, // 1 hour ago
        { id: '3', title: 'Reminder', message: 'Your meeting with Jane Smith is at 3:00 PM today.', timestamp: new Date(Date.now() - 7200000).toLocaleString() }, // 2 hours ago
        { id: '4', title: 'New Comment', message: 'Someone commented on your post.', timestamp: new Date(Date.now() - 10800000).toLocaleString() }, // 3 hours ago
      ];
      setNotifications(mockNotifications);
    } catch (err) {
      // In a real application, you'd handle different error types
      setError('Failed to fetch notifications. Please try again later.');
      console.error('API call error:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Fetch notifications only when the drawer opens
  useEffect(() => {
    if (open) {
      fetchNotifications();
    }
  }, [open, fetchNotifications]);

  // Determine drawer size based on `isMobile`
  const drawerSize = isMobile ? 'full' : 'sm'; // 'full' for mobile, 'sm' for desktop

  return (
    <Drawer
      open={open}
      onClose={onClose}
      placement="right"
      size={drawerSize}
      backdrop={true} // Allows clicking outside to close
      keyboard={true} // Allows closing with ESC key
    >
      <Drawer.Header>
        <Drawer.Title>Notifications</Drawer.Title>
        <Drawer.Actions>
          <Button onClick={onClose} appearance="subtle">Close</Button>
        </Drawer.Actions>
      </Drawer.Header>
      <Drawer.Body>
        {loading ? (
          <Loader center content="Loading notifications..." />
        ) : error ? (
          <Message type="error" header="Error">
            {error}
            <Button appearance="link" onClick={fetchNotifications} style={{ marginLeft: 10 }}>Retry</Button>
          </Message>
        ) : notifications.length === 0 ? (
          <Placeholder.Paragraph rows={3}>
            <p>No new notifications.</p>
          </Placeholder.Paragraph>
        ) : (
          <div>
            {notifications.map(notification => (
              <div key={notification.id} style={{ marginBottom: '15px', padding: '10px', border: '1px solid #e0e0e0', borderRadius: '4px' }}>
                <h4>{notification.title}</h4>
                <p>{notification.message}</p>
                <small style={{ color: '#888' }}>{notification.timestamp}</small>
              </div>
            ))}
            <Divider />
            <Button appearance="primary" block>View All Notifications</Button>
          </div>
        )}
      </Drawer.Body>
    </Drawer>
  );
};

export default NotificationsDrawer;