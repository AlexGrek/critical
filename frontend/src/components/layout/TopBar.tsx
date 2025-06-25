import React, { useState } from 'react';
import NotificationsDrawer from './NotificationsDrawer';
import { Bell, CircleUser, LogIn } from 'lucide-react';
import { useAuth } from '../user/AuthProvider';
import { useNavigate } from 'react-router-dom';

interface TopBarProps {
  currentAppName: string;
  isMenuOpen: boolean;
  setIsMenuOpen: (open: boolean) => void;
}

const TopBar: React.FC<TopBarProps> = ({
  currentAppName,
  isMenuOpen,
  setIsMenuOpen,
}) => {
  // Update page title when app name changes
  React.useEffect(() => {
    document.title = `${currentAppName} - BDSMtools`;
  }, [currentAppName]);

  const { user } = useAuth();
  const navigate = useNavigate();

  const [isNotifyDrawerOpen, setIsNotifyDrawerOpen] = useState(false);

  const handleUserClick = React.useCallback(() => {
    if (user != null) {
      console.log("user settings clicked");
    } else {
      navigate('/login')
    }
  }, [user, navigate])

  return (
    <div className="w-full h-12 flex items-center justify-between px-4 backdrop-blur-xl border-b border-gray-800/30 relative">
      <NotificationsDrawer
        open={isNotifyDrawerOpen}
        onClose={() => setIsNotifyDrawerOpen(false)}
      />
      {/* Left - App Name */}
      <div className="flex-1 min-w-0">
        <span className="text-gray-300 text-sm font-medium truncate block">
          {currentAppName}
        </span>
      </div>

      {/* Center - App Menu Button */}
      <div className="flex-shrink-0 mx-4">
        <button
          onClick={() => setIsMenuOpen(!isMenuOpen)}
          className="w-8 h-8 flex items-center justify-center rounded-full bg-transparent hover:bg-gray-800/40 transition-all duration-200 active:scale-95 relative overflow-hidden"
          aria-label="Open app menu"
        >
          <div
            className={`grid grid-cols-2 gap-0.5 transition-all duration-300 ${isMenuOpen
                ? 'transform translate-y-8 scale-150 opacity-0'
                : 'transform translate-y-0 scale-100 opacity-100'
              }`}
          >
            <div className="w-1.5 h-1.5 bg-gray-300 rounded-sm"></div>
            <div className="w-1.5 h-1.5 bg-gray-300 rounded-sm"></div>
            <div className="w-1.5 h-1.5 bg-gray-300 rounded-sm"></div>
            <div className="w-1.5 h-1.5 bg-gray-300 rounded-sm"></div>
          </div>
        </button>
      </div>

      {/* Right - User Actions */}
      <div className="flex-1 flex justify-end items-center gap-2">
        {/* Notifications Button */}
        <button
          onClick={() => setIsNotifyDrawerOpen(true)}
          className="w-8 h-8 flex items-center justify-center rounded-full bg-transparent hover:bg-gray-800/40 transition-all duration-200 active:scale-95"
          aria-label="Notifications"
        >
          <Bell />
        </button>

        {/* User/Settings Button */}
        <button
          className="w-8 h-8 flex items-center justify-center rounded-full bg-transparent hover:bg-gray-800/40 transition-all duration-200 active:scale-95"
          aria-label="User settings"
          onClick={handleUserClick}
        >
          {user == null ? <LogIn /> : <CircleUser />}
        </button>
      </div>
    </div>
  );
};

export default TopBar;