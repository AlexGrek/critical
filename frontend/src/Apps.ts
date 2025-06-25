import {
  Grid3X3, Settings, Camera, Mail, Calendar, Music,
  FileText, Calculator, MapPin, MessageCircle, Users
} from 'lucide-react';

export interface App {
  id: string;
  name: string;
  description: string;
  route: string;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
  popularity: number;
  lastUsed: Date;
}

const RAW_APPS: App[] = [
  {
    id: 'dashboard',
    name: 'Dashboard',
    description: 'Overview and analytics',
    route: '/dashboard',
    icon: Grid3X3,
    color: 'from-blue-500 to-blue-600',
    popularity: 95,
    lastUsed: new Date('2024-01-10'),
  },
  {
    id: 'login',
    name: 'Log In',
    description: 'Log in or register',
    route: '/login',
    icon: Users,
    color: 'from-gray-500 to-gray-600',
    popularity: 87,
    lastUsed: new Date('2024-01-09'),
  },
  {
    id: 'settings',
    name: 'Settings',
    description: 'App preferences',
    route: '/settings',
    icon: Settings,
    color: 'from-slate-500 to-slate-600',
    popularity: 65,
    lastUsed: new Date('2024-01-08'),
  },
  {
    id: 'camera',
    name: 'Camera',
    description: 'Photo and video capture',
    route: '/camera',
    icon: Camera,
    color: 'from-emerald-500 to-emerald-600',
    popularity: 78,
    lastUsed: new Date('2024-01-11'),
  },
  {
    id: 'mail',
    name: 'Mail',
    description: 'Email management',
    route: '/mail',
    icon: Mail,
    color: 'from-blue-400 to-blue-500',
    popularity: 92,
    lastUsed: new Date('2024-01-12'),
  },
  {
    id: 'calendar',
    name: 'Calendar',
    description: 'Schedule and events',
    route: '/calendar',
    icon: Calendar,
    color: 'from-red-500 to-red-600',
    popularity: 85,
    lastUsed: new Date('2024-01-07'),
  },
  {
    id: 'music',
    name: 'Music',
    description: 'Audio player',
    route: '/music',
    icon: Music,
    color: 'from-purple-500 to-purple-600',
    popularity: 73,
    lastUsed: new Date('2024-01-06'),
  },
  {
    id: 'documents',
    name: 'Documents',
    description: 'File management',
    route: '/documents',
    icon: FileText,
    color: 'from-orange-500 to-orange-600',
    popularity: 68,
    lastUsed: new Date('2024-01-05'),
  },
  {
    id: 'calculator',
    name: 'Calculator',
    description: 'Quick calculations',
    route: '/calculator',
    icon: Calculator,
    color: 'from-indigo-500 to-indigo-600',
    popularity: 55,
    lastUsed: new Date('2024-01-04'),
  },
  {
    id: 'personal',
    name: 'Personal page',
    description: 'Your personal space',
    route: '/personal',
    icon: MapPin,
    color: 'from-green-500 to-green-600',
    popularity: 62,
    lastUsed: new Date('2024-01-03'),
  },
  {
    id: 'messages',
    name: 'Messages',
    description: 'Chat and messaging',
    route: '/messages',
    icon: MessageCircle,
    color: 'from-cyan-500 to-cyan-600',
    popularity: 89,
    lastUsed: new Date('2024-01-02'),
  },
];

const STORAGE_KEY = 'appsLastUsed';

export function loadAppUsage(): Record<string, number> {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return {};
  try {
    return JSON.parse(raw) as Record<string, number>;
  } catch {
    return {};
  }
}

export function saveAppUsage(apps: App[]): void {
  const usage: Record<string, number> = {};
  for (const app of apps) {
    usage[app.id] = app.lastUsed.getTime(); // Save as timestamp
  }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(usage));
}

export function getApps(): App[] {
  const usage = loadAppUsage();
  return RAW_APPS.map((app) => ({
    ...app,
    lastUsed: new Date(usage[app.id] ?? new Date(app.lastUsed).getTime()),
  }));
}

const defaultApps = getApps();
export default defaultApps;
