import { Sun, Moon, Palette, Check, Maximize, Monitor, Server } from 'lucide-react'
import { motion } from 'framer-motion'
import { cn } from '../../utils'
import { ACCENT_COLORS, AppSettings } from '../../hooks/useAppSettings'
import { Section, Checkbox } from './SettingsShared'

interface AppearanceSettingsProps {
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
  activeTab: 'appearance' | 'interface';
}

export function AppearanceSettings({ settings, updateSettings, activeTab }: AppearanceSettingsProps) {
  if (activeTab === 'appearance') {
    return (
      <div className="space-y-8">
        <Section title="Theme" icon={Sun}>
          <div className="flex gap-4 p-1.5 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5">
            <button
              onClick={() => updateSettings({ theme: 'light' })}
              className={cn(
                "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl transition-all duration-200",
                settings.theme === 'light'
                  ? "bg-white dark:bg-gray-800 text-primary shadow-xl"
                  : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
              )}
            >
              <Sun size={18} />
              <span className="font-bold text-sm">Light</span>
            </button>
            <button
              onClick={() => updateSettings({ theme: 'dark' })}
              className={cn(
                "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl transition-all duration-200",
                settings.theme === 'dark'
                  ? "bg-white dark:bg-gray-800 text-primary shadow-xl"
                  : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
              )}
            >
              <Moon size={18} />
              <span className="font-bold text-sm">Dark</span>
            </button>
          </div>
        </Section>

        <Section title="Accent Color" icon={Palette}>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
            {ACCENT_COLORS.map((color) => (
              <button
                key={color.name}
                onClick={() => updateSettings({ accent_color: color.name })}
                className={cn(
                  "flex items-center gap-3 px-4 py-3 rounded-xl border transition-all duration-200 group relative overflow-hidden",
                  settings.accent_color === color.name
                    ? "bg-primary/10 border-primary shadow-glow-primary/20"
                    : "bg-black/5 dark:bg-white/5 border-black/5 dark:border-white/5 hover:border-primary/50"
                )}
              >
                <div
                  className="w-4 h-4 rounded-full shadow-sm shrink-0"
                  style={{ backgroundColor: `hsl(${color.value})` }}
                />
                <span className={cn(
                  "text-sm font-bold transition-colors",
                  settings.accent_color === color.name ? "text-primary" : "text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-200"
                )}>
                  {color.name}
                </span>

                {settings.accent_color === color.name && (
                  <div className="ml-auto">
                    <Check size={16} className="text-primary" />
                  </div>
                )}
              </button>
            ))}
          </div>
        </Section>

        <Section title="App Scaling" icon={Maximize}>
          <div className="p-6 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5">
            <div className="flex items-center justify-between mb-6">
              <div className="text-sm font-semibold text-gray-700 dark:text-gray-200">Scale the user interface</div>
              <span className="text-sm font-bold text-primary bg-primary/10 px-3 py-1 rounded-full">
                {Math.round(settings.scaling * 100)}%
              </span>
            </div>
            <input
              type="range"
              min="0.7"
              max="1.3"
              step="0.05"
              value={settings.scaling}
              onChange={(e) => updateSettings({ scaling: parseFloat(e.target.value) })}
              className="w-full h-2 bg-black/10 dark:bg-white/10 rounded-lg appearance-none cursor-pointer accent-primary hover:accent-primary/80 transition-all"
            />
            <div className="flex justify-between mt-4 px-1 text-[10px] font-bold text-gray-400 uppercase tracking-widest">
              <span>70%</span>
              <span>100%</span>
              <span>130%</span>
            </div>
          </div>
        </Section>

        <Section title="Console" icon={Monitor}>
          <div className="p-2 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5">
            <Checkbox
              label="White Console Text"
              description="Use white text for console logs by default instead of accent color"
              checked={settings.use_white_console_text}
              onChange={(val) => updateSettings({ use_white_console_text: val })}
            />
          </div>
        </Section>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <Section title="Display Options" icon={Monitor}>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
          <Checkbox
            label="Display IPv6"
            description="Use IPv6 addresses instead of IPv4 where available"
            checked={settings.display_ipv6}
            onChange={(val) => updateSettings({ display_ipv6: val })}
          />
          <Checkbox
            label="Hide IP Address"
            description="Mask your server's IP address in the UI"
            checked={settings.hide_ip_address}
            onChange={(val) => updateSettings({ hide_ip_address: val })}
          />
        </div>
      </Section>

      <Section title="Server Tabs" icon={Server}>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="space-y-2">
            <Checkbox
              label="Display server icon"
              checked={settings.display_server_icon}
              onChange={(val) => updateSettings({ display_server_icon: val })}
            />
            <Checkbox
              label="Display player count"
              checked={settings.display_online_player_count}
              onChange={(val) => updateSettings({ display_online_player_count: val })}
            />
            <Checkbox
              label="Display server version"
              checked={settings.display_server_version}
              onChange={(val) => updateSettings({ display_server_version: val })}
            />
            <Checkbox
              label="Display server status"
              checked={settings.display_server_status}
              onChange={(val) => updateSettings({ display_server_status: val })}
            />
            <Checkbox
              label="Navigational buttons"
              checked={settings.display_navigational_buttons}
              onChange={(val) => updateSettings({ display_navigational_buttons: val })}
            />
          </div>

          <div className="flex flex-col justify-center">
            <div className="text-[10px] font-bold text-gray-400 uppercase tracking-widest mb-2 text-center">Preview</div>
            <div className="p-4 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5 flex items-center gap-4">
              {settings.display_server_icon && (
                <div className="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center shrink-0">
                  <Server className="text-primary w-6 h-6" />
                </div>
              )}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-bold text-sm truncate">Survival Server</span>
                  {settings.display_server_status && (
                    <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]" />
                  )}
                </div>
                <div className="flex items-center gap-2 text-[10px] text-gray-500 font-medium">
                  {settings.display_server_version && <span>1.20.1</span>}
                  {settings.display_server_version && settings.display_online_player_count && <span>•</span>}
                  {settings.display_online_player_count && <span>12/20 Players</span>}
                </div>
              </div>
              {settings.display_navigational_buttons && (
                <div className="flex gap-1">
                  <div className="w-6 h-6 bg-black/10 dark:bg-white/10 rounded-md" />
                  <div className="w-6 h-6 bg-black/10 dark:bg-white/10 rounded-md" />
                </div>
              )}
            </div>
          </div>
        </div>
      </Section>
    </div>
  );
}
