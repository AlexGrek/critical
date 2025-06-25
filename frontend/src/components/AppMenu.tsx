// Updated AppMenu.tsx
import React, { useState, useEffect, useRef } from 'react';
import { X, Search, ArrowUpDown } from 'lucide-react';
import OsDropdown from './ui/OsDropdown';
import './AppMenu.css';
import { getApps, saveAppUsage, type App } from '../Apps';

interface SortingMode {
  id: string;
  label: string;
  description: string;
}

interface AppMenuProps {
  isOpen: boolean;
  onClose: () => void;
  navigate?: (route: string) => void;
  currentPath: string;
  onAppChanged?: (app: App) => void;
}

const sortingModes: SortingMode[] = [
  { id: 'custom', label: 'Custom Order', description: 'Default arrangement' },
  { id: 'popular', label: 'Most Popular', description: 'By usage frequency' },
  { id: 'recent', label: 'Recently Used', description: 'By last accessed' },
  { id: 'alphabetical', label: 'A to Z', description: 'Alphabetical order' }
];

const AppMenu: React.FC<AppMenuProps> = ({ isOpen, onClose, navigate, currentPath, onAppChanged }) => {
  const [searchTerm, setSearchTerm] = useState<string>('');
  const [appsState, setAppsState] = useState<App[]>([]);
  const [currentApp, setCurrentApp] = useState<string | null>(null);
  const [filteredApps, setFilteredApps] = useState<App[]>([]);
  const [sortedApps, setSortedApps] = useState<App[]>([]);
  const [isVisible, setIsVisible] = useState<boolean>(false);
  const [isAnimating, setIsAnimating] = useState<boolean>(false);
  const [sortMode, setSortMode] = useState<string>('custom');
  const [showSortDropdown, setShowSortDropdown] = useState<boolean>(false);
  const sortButtonRef = useRef<HTMLButtonElement>(null!);
  const appMenuRef = useRef<HTMLDivElement>(null!);

  useEffect(() => {
    const hydrated = getApps();
    setAppsState(hydrated);
  }, []);

  useEffect(() => {
    if (isOpen) {
      setIsVisible(true);
      setTimeout(() => setIsAnimating(true), 10);
    } else {
      setIsAnimating(false);
      setShowSortDropdown(false);
      setTimeout(() => setIsVisible(false), 500);
    }
  }, [isOpen]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (appMenuRef.current && !appMenuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen, onClose]);

  useEffect(() => {
    const sorted = [...appsState];
    switch (sortMode) {
      case 'popular':
        sorted.sort((a, b) => b.popularity - a.popularity);
        break;
      case 'recent':
        sorted.sort((a, b) => b.lastUsed.getTime() - a.lastUsed.getTime());
        break;
      case 'alphabetical':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      default:
        break;
    }
    setSortedApps(sorted);
  }, [sortMode, appsState]);

  useEffect(() => {
    if (searchTerm) {
      const filtered = sortedApps.filter(app =>
        app.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        app.description.toLowerCase().includes(searchTerm.toLowerCase())
      );
      setFilteredApps(filtered);
    } else {
      setFilteredApps(sortedApps);
    }
  }, [searchTerm, sortedApps]);

  useEffect(() => {
    const matched = appsState.find(app => currentPath.startsWith(app.route));
    if (matched && (currentApp != matched.id)) {
      setCurrentApp(matched.id);
      const now = new Date();
      const updated = appsState.map(app =>
        app.id === matched.id ? { ...app, lastUsed: now } : app
      );
      setAppsState(updated);
      saveAppUsage(updated);
      onAppChanged?.(matched);
    }
  }, [appsState, currentApp, currentPath, onAppChanged]);

  const handleAppClick = (route: string): void => {
    const now = new Date();
    const updated = appsState.map(app =>
      app.route === route ? { ...app, lastUsed: now } : app
    );
    setAppsState(updated);
    saveAppUsage(updated);
    navigate?.(route);
    onClose();
  };

  const handleSortModeChange = (mode: string): void => {
    setSortMode(mode);
    localStorage.setItem('appMenuSortMode', mode);
    setShowSortDropdown(false);
  };

  const dropdownItems = sortingModes.map((mode) => ({
    id: mode.id,
    label: mode.label,
    description: mode.description,
    onClick: () => handleSortModeChange(mode.id),
    isSelected: sortMode === mode.id,
  }));

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center md:items-center md:pt-0 pt-8" style={{ perspective: '1000px' }}>
      <div className={`absolute inset-0 bg-black/40 transition-opacity duration-500 ease-out ${isAnimating ? 'opacity-100' : 'opacity-0'} app-menu-backdrop-custom-blur`} />

      <div
        ref={appMenuRef}
        className={`relative w-full h-full md:w-[500px] md:h-[600px] md:rounded-2xl bg-gray-900/90 border border-gray-700/30 shadow-2xl overflow-hidden flex flex-col ${isAnimating ? 'app-menu-container-animate-in' : 'app-menu-container-animate-out'} app-menu-container-custom-blur app-menu-container-transition-transform`}
      >
        <div className="flex items-center justify-between p-4 border-b border-gray-700/20 flex-shrink-0">
          <h2 className="text-lg font-semibold text-gray-100">Applications</h2>
          <button onClick={onClose} className="p-2 rounded-full hover:bg-white/5 transition-colors duration-200" aria-label="Close menu">
            <X className="w-5 h-5 text-gray-300" />
          </button>
        </div>

        <div className="p-4 border-b border-gray-700/20 flex-shrink-0">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search applications..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 bg-gray-800/50 rounded-lg border border-gray-600/30 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-transparent text-gray-100 placeholder-gray-400 transition-all duration-200"
              />
            </div>

            <div className="relative">
              <button
                ref={sortButtonRef}
                onClick={() => setShowSortDropdown(!showSortDropdown)}
                className="p-2 bg-gray-800/50 rounded-lg border border-gray-600/30 hover:bg-gray-700/60 hover:border-gray-500/40 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-transparent text-gray-300 transition-all duration-200"
                aria-label="Sort options"
              >
                <ArrowUpDown className="w-4 h-4" />
              </button>
              <OsDropdown
                items={dropdownItems}
                isOpen={showSortDropdown}
                onClose={() => setShowSortDropdown(false)}
                triggerRef={sortButtonRef}
              />
            </div>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-4 overscroll-contain">
          <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
            {filteredApps.map((app) => {
              const IconComponent = app.icon;
              return (
                <button
                  key={app.id}
                  onClick={() => handleAppClick(app.route)}
                  className="group p-4 rounded-xl bg-gray-800/40 hover:bg-gray-700/60 border border-gray-600/20 hover:border-gray-500/40 transition-all duration-300 hover:scale-105 hover:shadow-lg text-left focus:outline-none focus:ring-2 focus:ring-blue-500/50 transform-gpu"
                  style={{ animation: isAnimating ? 'slideInUp 0.1s ease-out forwards' : 'none' }}
                >
                  <div className="flex flex-col items-center text-center space-y-2">
                    <div className={`w-12 h-12 rounded-xl bg-gradient-to-br ${app.color} flex items-center justify-center shadow-lg group-hover:shadow-xl transition-all duration-300 group-hover:scale-110 transform-gpu`}>
                      <IconComponent className="w-6 h-6 text-white" />
                    </div>
                    <h3 className="font-medium text-gray-100 text-sm leading-tight">{app.name}</h3>
                    <p className="text-xs text-gray-300 leading-tight opacity-80">{app.description}</p>
                  </div>
                </button>
              );
            })}
          </div>

          {filteredApps.length === 0 && (
            <div className="text-center py-8">
              <div className="text-gray-400 text-sm">No applications found</div>
              <div className="text-gray-500 text-xs mt-1">Try a different search term</div>
            </div>
          )}
        </div>

        <div className="p-2 border-t border-gray-700/20 bg-gray-800/20 flex-shrink-0">
          <div className="text-xs text-gray-400 text-center">
            {filteredApps.length} application{filteredApps.length !== 1 ? 's' : ''} available
            {sortMode !== 'custom' && (
              <span className="ml-2 text-gray-500">• Sorted by {sortingModes.find(m => m.id === sortMode)?.label.toLowerCase()}</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default AppMenu;
