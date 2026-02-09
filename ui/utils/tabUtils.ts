import {
  Settings,
  LayoutDashboard,
  Users,
  Terminal,
  Puzzle,
  Layers,
  History,
  Calendar,
  FileText,
  Sliders,
  BarChart3
} from 'lucide-react'
import { TabId } from '../types'

export const ALL_TABS: { id: TabId; label: string; icon: any }[] = [
  { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
  { id: 'console', label: 'Console', icon: Terminal },
  { id: 'logs', label: 'Logs', icon: FileText },
  { id: 'stats', label: 'Statistics', icon: BarChart3 },
  { id: 'players', label: 'Players', icon: Users },
  { id: 'config', label: 'Config', icon: Sliders },
  { id: 'plugins', label: 'Plugins', icon: Puzzle },
  { id: 'mods', label: 'Mods', icon: Layers },
  { id: 'backups', label: 'Backups', icon: History },
  { id: 'scheduler', label: 'Scheduler', icon: Calendar },
  { id: 'settings', label: 'Settings', icon: Settings },
];

export const supportsPlugins = (loader?: string) => {
  if (!loader) return false;
  const l = loader.toLowerCase();
  return ['paper', 'purpur', 'spigot', 'bukkit', 'velocity'].includes(l);
};

export const supportsMods = (loader?: string) => {
  if (!loader) return false;
  const l = loader.toLowerCase();
  return ['fabric', 'forge', 'neoforge', 'quilt'].includes(l);
};

export const getAvailableTabs = (modLoader?: string) => {
  return ALL_TABS.filter(tab => {
    if (tab.id === 'plugins') return supportsPlugins(modLoader);
    if (tab.id === 'mods') return supportsMods(modLoader);
    return true;
  });
};
