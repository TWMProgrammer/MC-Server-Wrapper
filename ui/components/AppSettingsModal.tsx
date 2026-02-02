import { X, Settings, Palette, Check, Moon, Sun, Maximize, Layout, Users, Monitor, Power, Globe, Server, ChevronRight } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useState } from 'react'
import { cn } from '../utils'
import { ACCENT_COLORS, AppSettings } from '../hooks/useAppSettings'
import { Select } from './Select'

interface AppSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
}

type SettingsTab = 'general' | 'appearance' | 'interface' | 'players';

interface TabItem {
  id: SettingsTab;
  label: string;
  icon: any;
  description: string;
}

const TABS: TabItem[] = [
  { id: 'general', label: 'General', icon: Settings, description: 'Basic application behavior' },
  { id: 'appearance', label: 'Appearance', icon: Palette, description: 'Themes and visual styling' },
  { id: 'interface', label: 'Interface', icon: Layout, description: 'UI elements and layout' },
  { id: 'players', label: 'Players', icon: Users, description: 'Player skin and data settings' },
];

function SidebarItem({
  tab,
  isActive,
  onClick
}: {
  tab: TabItem;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-3 p-3 rounded-xl transition-all duration-200 group relative",
        isActive
          ? "bg-primary/10 text-primary shadow-sm shadow-primary/5"
          : "text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5"
      )}
    >
      <div className={cn(
        "p-2 rounded-lg transition-colors",
        isActive ? "bg-primary text-white" : "bg-black/5 dark:bg-white/5 group-hover:bg-black/10 dark:group-hover:bg-white/10"
      )}>
        <tab.icon size={18} />
      </div>
      <div className="flex-1 text-left">
        <div className="text-sm font-bold leading-none">{tab.label}</div>
        <div className="text-[10px] text-gray-500 mt-1 line-clamp-1">{tab.description}</div>
      </div>
      {isActive && (
        <motion.div
          layoutId="activeTab"
          className="absolute left-0 w-1 h-6 bg-primary rounded-full"
        />
      )}
      <ChevronRight size={14} className={cn(
        "transition-transform",
        isActive ? "opacity-100 translate-x-0" : "opacity-0 -translate-x-2"
      )} />
    </button>
  );
}

function Section({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) {
  return (
    <section className="space-y-4">
      <div className="flex items-center gap-2 px-1">
        <div className="p-1.5 bg-black/5 dark:bg-white/5 rounded-lg text-gray-500">
          <Icon size={16} />
        </div>
        <h3 className="text-sm font-bold text-gray-900 dark:text-white uppercase tracking-wider">{title}</h3>
      </div>
      <div className="space-y-4">
        {children}
      </div>
    </section>
  );
}

function Checkbox({
  label,
  checked,
  onChange,
  description,
  disabled = false
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
  disabled?: boolean;
}) {
  return (
    <label className={cn(
      "flex items-start gap-3 p-3 rounded-xl transition-all cursor-pointer group",
      disabled ? "opacity-50 cursor-not-allowed" : "hover:bg-black/5 dark:hover:bg-white/5"
    )}>
      <div className="relative flex items-center mt-0.5">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => !disabled && onChange(e.target.checked)}
          className="peer sr-only"
          disabled={disabled}
        />
        <div className={cn(
          "w-5 h-5 rounded-md border-2 transition-all flex items-center justify-center",
          checked
            ? "bg-primary border-primary shadow-glow-primary/20"
            : "border-black/20 dark:border-white/20 group-hover:border-primary/50"
        )}>
          {checked && <Check size={14} className="text-white" />}
        </div>
      </div>
      <div className="flex-1">
        <div className={cn(
          "text-sm font-semibold transition-colors",
          checked ? "text-primary" : "text-gray-700 dark:text-gray-200"
        )}>
          {label}
        </div>
        {description && (
          <div className="text-xs text-gray-500 mt-0.5 leading-relaxed">
            {description}
          </div>
        )}
      </div>
    </label>
  )
}

export function AppSettingsModal({
  isOpen,
  onClose,
  settings,
  updateSettings
}: AppSettingsModalProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="absolute inset-0 bg-black/80 backdrop-blur-md"
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            className="relative w-full max-w-4xl bg-white dark:bg-gray-950 border border-black/10 dark:border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col h-[90vh]"
          >
            <div className="flex flex-1 overflow-hidden">
              {/* Sidebar */}
              <div className="w-64 bg-black/[0.02] dark:bg-white/[0.02] border-r border-black/5 dark:border-white/5 flex flex-col shrink-0">
                <div className="p-6 border-b border-black/5 dark:border-white/5">
                  <div className="flex items-center gap-3">
                    <div className="p-2 bg-primary/20 rounded-lg">
                      <Settings className="text-primary w-5 h-5" />
                    </div>
                    <div>
                      <h2 className="text-lg font-bold text-gray-900 dark:text-white leading-tight">Settings</h2>
                      <p className="text-[10px] text-gray-500 uppercase tracking-wider font-bold">Preferences</p>
                    </div>
                  </div>
                </div>

                <div className="flex-1 p-4 space-y-2 overflow-y-auto">
                  {TABS.map((tab) => (
                    <SidebarItem
                      key={tab.id}
                      tab={tab}
                      isActive={activeTab === tab.id}
                      onClick={() => setActiveTab(tab.id)}
                    />
                  ))}
                </div>

                <div className="p-4 mt-auto">
                  <div className="p-4 bg-primary/5 rounded-2xl border border-primary/10">
                    <div className="text-[10px] font-bold text-primary uppercase tracking-widest mb-1">Version</div>
                    <div className="text-sm font-bold text-gray-700 dark:text-gray-300">1.0.0-beta</div>
                  </div>
                </div>
              </div>

              {/* Content Area */}
              <div className="flex-1 flex flex-col bg-white dark:bg-gray-950 min-w-0">
                {/* Header */}
                <div className="h-16 px-8 border-b border-black/5 dark:border-white/5 flex items-center justify-between shrink-0">
                  <div>
                    <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                      {TABS.find(t => t.id === activeTab)?.label}
                    </h3>
                  </div>
                  <button
                    onClick={onClose}
                    className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg transition-colors text-gray-400 hover:text-gray-900 dark:hover:text-white"
                  >
                    <X size={20} />
                  </button>
                </div>

                {/* Main Content */}
                <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                  <AnimatePresence mode="wait">
                    <motion.div
                      key={activeTab}
                      initial={{ opacity: 0, x: 10 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -10 }}
                      transition={{ duration: 0.2 }}
                      className="space-y-8"
                    >
                      {activeTab === 'general' && (
                        <>
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
                          </Section>
                        </>
                      )}

                      {activeTab === 'appearance' && (
                        <>
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
                                  {Math.round((settings.scaling + 0.2) * 100)}%
                                </span>
                              </div>
                              <input
                                type="range"
                                min="0.5"
                                max="1.1"
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
                        </>
                      )}

                      {activeTab === 'interface' && (
                        <>
                          <Section title="Display Options" icon={Monitor}>
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                              <Checkbox
                                label="Dynamic graph scaling"
                                description="Automatically adjust graph Y-axis based on current data"
                                checked={settings.dynamic_graph_scaling}
                                onChange={(val) => updateSettings({ dynamic_graph_scaling: val })}
                              />
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
                        </>
                      )}

                      {activeTab === 'players' && (
                        <>
                          <Section title="Player List" icon={Users}>
                            <div className="space-y-4 p-4 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5">
                              <Checkbox
                                label="Download player heads"
                                description="Fetch and display player skins in the player list"
                                checked={settings.download_player_heads}
                                onChange={(val) => updateSettings({ download_player_heads: val })}
                              />
                              <div className={cn("pl-8 space-y-2 transition-all", !settings.download_player_heads && "opacity-50 pointer-events-none")}>
                                <Checkbox
                                  label="Use helm heads"
                                  description="Display the 3D helm layer for player skins"
                                  checked={settings.use_helm_heads}
                                  onChange={(val) => updateSettings({ use_helm_heads: val })}
                                  disabled={!settings.download_player_heads}
                                />
                                <Checkbox
                                  label="Query heads by username"
                                  description="Use usernames instead of UUIDs for head lookups"
                                  checked={settings.query_heads_by_username}
                                  onChange={(val) => updateSettings({ query_heads_by_username: val })}
                                  disabled={!settings.download_player_heads}
                                />
                              </div>
                            </div>
                          </Section>
                        </>
                      )}
                    </motion.div>
                  </AnimatePresence>
                </div>

                {/* Footer */}
                <div className="p-6 bg-black/[0.05] dark:bg-white/[0.02] border-t border-black/5 dark:border-white/5 flex justify-end shrink-0">
                  <button
                    onClick={onClose}
                    className="px-8 py-2.5 bg-primary text-white rounded-xl font-bold shadow-glow-primary hover:shadow-primary/40 transition-all duration-200 active:scale-95"
                  >
                    Done
                  </button>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}
