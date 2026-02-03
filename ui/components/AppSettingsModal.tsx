import { X, Settings, Palette, Layout, Users, ChevronRight } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useState } from 'react'
import { cn } from '../utils'
import { AppSettings } from '../hooks/useAppSettings'
import { AppearanceSettings } from './settings/AppearanceSettings'
import { BehaviorSettings } from './settings/BehaviorSettings'
import { SystemSettings } from './settings/SystemSettings'

interface AppSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
}

type SettingsTab = 'general' | 'appearance' | 'interface' | 'players' | 'java';

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
  { id: 'java', label: 'Java', icon: Settings, description: 'Manage Java versions' },
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
                      {(activeTab === 'appearance' || activeTab === 'interface') && (
                        <AppearanceSettings
                          settings={settings}
                          updateSettings={updateSettings}
                          activeTab={activeTab}
                        />
                      )}

                      {(activeTab === 'general' || activeTab === 'players') && (
                        <BehaviorSettings
                          settings={settings}
                          updateSettings={updateSettings}
                          activeTab={activeTab}
                        />
                      )}

                      {activeTab === 'java' && (
                        <SystemSettings
                          settings={settings}
                          updateSettings={updateSettings}
                        />
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
