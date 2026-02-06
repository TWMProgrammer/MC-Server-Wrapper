import { Layout, Power, Globe, Monitor, Users } from 'lucide-react'
import { cn } from '../../utils'
import { AppSettings } from '../../hooks/useAppSettings'
import { Select } from '../Select'
import { Section, Checkbox } from './SettingsShared'

interface BehaviorSettingsProps {
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
  activeTab: 'general' | 'players';
}

export function BehaviorSettings({ settings, updateSettings, activeTab }: BehaviorSettingsProps) {
  if (activeTab === 'general') {
    return (
      <div className="space-y-8">
        <Section title="Navigation" icon={Layout}>
          <div className="p-4 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5">
            <div className="flex items-center justify-between gap-4">
              <div>
                <div className="text-sm font-semibold text-gray-700 dark:text-gray-200">Start page on launch</div>
                <div className="text-xs text-gray-500 mt-1">Choose which page opens when you start the application</div>
              </div>
              <div className="w-48">
                <Select
                  value={settings.start_page}
                  onChange={(val) => updateSettings({ start_page: val })}
                  options={[
                    { label: 'Dashboard', value: 'Dashboard' },
                    { label: 'Global Dashboard', value: 'Global Dashboard' },
                    { label: 'Instances', value: 'Instances' },
                  ]}
                />
              </div>
            </div>
          </div>
        </Section>

        <Section title="Close Preference" icon={Power}>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            {[
              { id: 'HideToSystemTray', label: 'System Tray', desc: 'Minimize to tray icon', icon: Globe },
              { id: 'HideToTaskbar', label: 'Taskbar', desc: 'Minimize to taskbar', icon: Monitor },
              { id: 'Exit', label: 'Exit Application', desc: 'Close application', icon: Power },
            ].map((pref) => (
              <button
                key={pref.id}
                onClick={() => updateSettings({ close_behavior: pref.id as any })}
                className={cn(
                  "flex flex-col items-center gap-3 p-4 rounded-2xl border transition-all text-center group",
                  settings.close_behavior === pref.id
                    ? "bg-primary/10 border-primary shadow-glow-primary/20"
                    : "bg-black/5 dark:bg-white/5 border-black/5 dark:border-white/5 hover:border-primary/30"
                )}
              >
                <div className={cn(
                  "p-3 rounded-xl transition-colors",
                  settings.close_behavior === pref.id ? "bg-primary text-white" : "bg-black/10 dark:bg-white/10 text-gray-400 group-hover:text-primary"
                )}>
                  <pref.icon size={20} />
                </div>
                <div>
                  <div className={cn(
                    "text-sm font-bold",
                    settings.close_behavior === pref.id ? "text-primary" : "text-gray-700 dark:text-gray-200"
                  )}>{pref.label}</div>
                  <div className="text-[10px] text-gray-500 mt-0.5">{pref.desc}</div>
                </div>
              </button>
            ))}
          </div>

          {settings.close_behavior === 'HideToSystemTray' && (
            <div className="mt-4 pt-4 border-t border-black/5 dark:border-white/5">
              <Checkbox
                label="Show tray notification"
                description="Notify when the app is minimized to the system tray"
                checked={settings.show_tray_notification}
                onChange={(val) => updateSettings({ show_tray_notification: val })}
              />
            </div>
          )}
        </Section>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <Section title="Player Heads" icon={Users}>
        <div className="space-y-2">
          <Checkbox
            label="Download player heads"
            description="Automatically download player skins for the player list"
            checked={settings.download_player_heads}
            onChange={(val) => updateSettings({ download_player_heads: val })}
          />
          <Checkbox
            label="Use helm heads"
            description="Show the player's helmet layer in the head icon"
            checked={settings.use_helm_heads}
            onChange={(val) => updateSettings({ use_helm_heads: val })}
            disabled={!settings.download_player_heads}
          />
          <Checkbox
            label="Query heads by UUID"
            description="Use UUIDs instead of usernames for head lookups"
            checked={settings.query_heads_by_uuid}
            onChange={(val) => updateSettings({ query_heads_by_uuid: val })}
            disabled={!settings.download_player_heads}
          />
        </div>
      </Section>
    </div>
  );
}
