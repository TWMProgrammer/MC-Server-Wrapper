import { useState, useEffect, createContext, useContext, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';

export type AccentColor = {
  name: string;
  value: string; // HSL value without 'hsl()' prefix, e.g. "221.2 83.2% 53.3%"
};

export const ACCENT_COLORS: AccentColor[] = [
  { name: 'Blue', value: '221.2 83.2% 53.3%' },
  { name: 'Emerald', value: '160 84% 39%' },
  { name: 'Rose', value: '346 84% 61%' },
  { name: 'Amber', value: '38 92% 50%' },
  { name: 'Indigo', value: '239 84% 67%' },
  { name: 'Violet', value: '262 83% 58%' },
];

export type Theme = 'dark' | 'light';
export type CloseBehavior = 'HideToSystemTray' | 'HideToTaskbar' | 'Exit';

export interface ManagedJavaVersion {
  id: string;
  name: string;
  path: string;
  version: string;
  major_version: number;
}

export interface AppSettings {
  // Interface
  display_ipv6: boolean;
  hide_ip_address: boolean;
  use_white_console_text: boolean;

  // Navigation
  start_page: string;

  // Player List
  download_player_heads: boolean;
  use_helm_heads: boolean;
  query_heads_by_username: boolean;

  // Server Tabs
  display_server_icon: boolean;
  display_online_player_count: boolean;
  display_server_version: boolean;
  display_server_status: boolean;
  display_navigational_buttons: boolean;

  // Close Preference
  close_behavior: CloseBehavior;
  show_tray_notification: boolean;

  // Appearance (Existing)
  accent_color: string;
  theme: Theme;
  scaling: number;

  // Java Management
  managed_java_versions: ManagedJavaVersion[];
}

const DEFAULT_SETTINGS: AppSettings = {
  display_ipv6: false,
  hide_ip_address: false,
  use_white_console_text: false,
  start_page: "Dashboard",
  download_player_heads: true,
  use_helm_heads: true,
  query_heads_by_username: false,
  display_server_icon: true,
  display_online_player_count: true,
  display_server_version: true,
  display_server_status: true,
  display_navigational_buttons: true,
  close_behavior: 'HideToSystemTray',
  show_tray_notification: true,
  accent_color: "Blue",
  theme: "dark",
  scaling: 0.8,
  managed_java_versions: [],
};

interface AppSettingsContextType {
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => Promise<void>;
  isLoading: boolean;
  appVersion: string;
  accentColor: AccentColor;
  setAccentColor: (color: AccentColor) => Promise<void>;
  theme: Theme;
  setTheme: (theme: Theme) => Promise<void>;
  scaling: number;
  setScaling: (scaling: number) => Promise<void>;
}

const AppSettingsContext = createContext<AppSettingsContextType | undefined>(undefined);

export function AppSettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isLoading, setIsLoading] = useState(true);
  const [appVersion, setAppVersion] = useState<string>('0.0.0');

  // Load settings from backend on mount
  useEffect(() => {
    // Load settings
    invoke<AppSettings>('get_app_settings')
      .then((loadedSettings) => {
        setSettings(loadedSettings);
        setIsLoading(false);
      })
      .catch((err) => {
        console.error('Failed to load app settings:', err);
        setIsLoading(false);
      });

    // Load version
    getVersion().then(setAppVersion).catch(err => {
      console.error('Failed to get app version:', err);
    });
  }, []);

  // Apply appearance settings
  useEffect(() => {
    const root = document.documentElement;
    const accent = ACCENT_COLORS.find(c => c.name === settings.accent_color) || ACCENT_COLORS[0];

    root.style.setProperty('--primary', accent.value);

    // Calculate hover and active colors (roughly)
    const [h, s, l] = accent.value.split(' ');
    const lValue = parseFloat(l.replace('%', ''));

    root.style.setProperty('--primary-hover', `${h} ${s} ${lValue - 8}%`);
    root.style.setProperty('--primary-active', `${h} ${s} ${lValue - 18}%`);

    if (settings.theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [settings.accent_color, settings.theme]);

  const updateSettings = async (newSettings: Partial<AppSettings>) => {
    const updated = { ...settings, ...newSettings };
    setSettings(updated);
    try {
      await invoke('update_app_settings', { settings: updated });
    } catch (err) {
      console.error('Failed to update app settings:', err);
    }
  };

  const contextValue: AppSettingsContextType = {
    settings,
    updateSettings,
    isLoading,
    appVersion,
    accentColor: ACCENT_COLORS.find(c => c.name === settings.accent_color) || ACCENT_COLORS[0],
    setAccentColor: (color: AccentColor) => updateSettings({ accent_color: color.name }),
    theme: settings.theme,
    setTheme: (theme: Theme) => updateSettings({ theme }),
    scaling: settings.scaling,
    setScaling: (scaling: number) => updateSettings({ scaling }),
  };

  return (
    <AppSettingsContext.Provider value={contextValue}>
      {children}
    </AppSettingsContext.Provider>
  );
}

export function useAppSettings() {
  const context = useContext(AppSettingsContext);
  if (context === undefined) {
    throw new Error('useAppSettings must be used within an AppSettingsProvider');
  }
  return context;
}
