import React, { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';

// Define the shape of an application
interface AppItem {
  id: string;
  name: string;
  description: string;
  logo: string; // URL to the app logo
  path: string; // React Router path
}

interface AppMenuDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

const apps: AppItem[] = [
  {
    id: '1',
    name: 'Finder',
    description: 'Browse your files and folders.',
    logo: 'https://via.placeholder.com/48/0000FF/FFFFFF?text=F', // Example logo
    path: '/finder',
  },
  {
    id: '2',
    name: 'Safari',
    description: 'The fastest way to browse the web.',
    logo: 'https://via.placeholder.com/48/FF0000/FFFFFF?text=S', // Example logo
    path: '/safari',
  },
  {
    id: '3',
    name: 'Mail',
    description: 'Stay connected with your emails.',
    logo: 'https://via.placeholder.com/48/00FF00/FFFFFF?text=M', // Example logo
    path: '/mail',
  },
  {
    id: '4',
    name: 'Photos',
    description: 'Organize and edit your photos.',
    logo: 'https://via.placeholder.com/48/FFFF00/000000?text=P', // Example logo
    path: '/photos',
  },
  {
    id: '5',
    name: 'Music',
    description: 'Listen to your favorite tunes.',
    logo: 'https://via.placeholder.com/48/800080/FFFFFF?text=Mu', // Example logo
    path: '/music',
  },
  {
    id: '6',
    name: 'Settings',
    description: 'Configure your system preferences.',
    logo: 'https://via.placeholder.com/48/FF8C00/FFFFFF?text=Se', // Example logo
    path: '/settings',
  },
  {
    id: '7',
    name: 'Messages',
    description: 'Send and receive messages.',
    logo: 'https://via.placeholder.com/48/00CED1/FFFFFF?text=Me', // Example logo
    path: '/messages',
  },
  {
    id: '8',
    name: 'Calendar',
    description: 'Manage your schedule and events.',
    logo: 'https://via.placeholder.com/48/FF6347/FFFFFF?text=C', // Example logo
    path: '/calendar',
  },
];

const AppMenuDrawer: React.FC<AppMenuDrawerProps> = ({ isOpen, onClose }) => {
  const navigate = useNavigate();
  const drawerRef = useRef<HTMLDivElement>(null);

  const handleAppClick = (path: string) => {
    navigate(path);
    onClose(); // Close the drawer after navigation
  };

  const handleKeyDown = React.useCallback((event: KeyboardEvent) => {
    if (event.key === 'Escape' && isOpen) {
      onClose();
    }
  }, [isOpen, onClose]);

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown, isOpen, onClose]);

  // Close drawer if clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (drawerRef.current && !drawerRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, onClose]);

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex justify-center items-center bg-black bg-opacity-20 backdrop-blur-sm"
      aria-modal="true"
      role="dialog"
    >
      <div
        ref={drawerRef}
        className="relative w-full h-full md:w-[500px] md:h-[400px]
                   bg-gray-800/80 backdrop-blur-xl border border-gray-700/50 rounded-lg shadow-2xl
                   flex flex-col overflow-hidden transform transition-all duration-300 ease-out
                   data-[state=open]:translate-y-0 data-[state=closed]:-translate-y-full"
        data-state={isOpen ? 'open' : 'closed'}
      >
        <div className="p-4 border-b border-gray-700/50 flex justify-between items-center">
          <h2 className="text-xl font-semibold text-white">Applications</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors duration-200"
            aria-label="Close menu"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4 p-4 overflow-y-auto custom-scrollbar">
          {apps.map((app) => (
            <div
              key={app.id}
              className="flex flex-col items-center p-3 rounded-lg cursor-pointer
                         hover:bg-gray-700/50 transition-colors duration-200
                         group"
              onClick={() => handleAppClick(app.path)}
            >
              <img
                src={app.logo}
                alt={`${app.name} logo`}
                className="w-12 h-12 rounded-lg object-cover mb-2 transform transition-transform duration-200 group-hover:scale-105"
              />
              <p className="text-white text-sm font-medium truncate w-full text-center group-hover:text-blue-400 transition-colors">
                {app.name}
              </p>
              <p className="text-gray-400 text-xs text-center line-clamp-2">
                {app.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default AppMenuDrawer;