import { Database, Plus, Sparkles, Settings, Server as ServerIcon } from 'lucide-react'
import { motion } from 'framer-motion'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Instance } from '../types'
import { cn } from '../utils'
import { useAppSettings, AppSettings } from '../hooks/useAppSettings'
import { InstanceSettingsDropdown } from '../InstanceSettingsDropdown'

interface SidebarProps {
  instances: Instance[];
  selectedInstanceId: string | null;
  onSelectInstance: (id: string | null) => void;
  onCreateNew: () => void;
  onOpenSettings: () => void;
  onInstancesUpdated: (id?: string) => void;
  settings: AppSettings;
}

export function Sidebar({
  instances,
  selectedInstanceId,
  onSelectInstance,
  onCreateNew,
  onOpenSettings,
  onInstancesUpdated,
}: Omit<SidebarProps, 'settings'>) {
  const { settings, appVersion } = useAppSettings();

  return (
    <div className="w-72 bg-sidebar-bg border-r border-black/5 dark:border-white/5 flex flex-col h-full shadow-2xl z-10 transition-colors duration-300">
      <div
        className="p-6 flex items-center gap-3 cursor-pointer hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
        onClick={() => onSelectInstance(null)}
      >
        <div className="p-2 bg-primary/20 rounded-xl">
          <Database className="text-primary w-6 h-6" />
        </div>
        <div>
          <h1 className="font-bold text-xl tracking-tight bg-clip-text text-transparent bg-gradient-to-br from-gray-900 to-gray-600 dark:from-white dark:to-gray-400 transition-all duration-300">
            MC Wrapper
          </h1>
          <div className="flex items-center gap-1 text-[10px] text-primary font-bold uppercase tracking-widest opacity-80">
            <Sparkles size={10} />
            <span>Premium</span>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-4 space-y-6 custom-scrollbar">
        <div>
          <div className="px-3 mb-3 text-[10px] font-bold text-gray-500 uppercase tracking-[0.2em]">
            Server Instances
          </div>
          <div className="space-y-1">
            {instances.map(inst => (
              <motion.button
                key={inst.id}
                whileHover={{ x: 4 }}
                whileTap={{ scale: 0.98 }}
                onClick={() => onSelectInstance(inst.id)}
                className={cn(
                  "w-full text-left px-4 py-3 rounded-xl transition-all duration-200 flex items-center gap-3 group relative overflow-hidden",
                  selectedInstanceId === inst.id
                    ? "bg-primary text-white shadow-glow-primary shadow-primary/20"
                    : "hover:bg-black/5 dark:hover:bg-white/[0.03] text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                )}
              >
                {selectedInstanceId === inst.id && (
                  <motion.div
                    layoutId="active-pill"
                    className="absolute left-0 w-1 h-6 bg-white rounded-r-full"
                  />
                )}

                {settings.display_server_icon && (
                  <div className="relative">
                    <div className={cn(
                      "w-8 h-8 rounded-lg bg-black/5 dark:bg-white/5 flex items-center justify-center text-gray-400 overflow-hidden",
                      selectedInstanceId === inst.id && "bg-white/20 text-white"
                    )}>
                      {inst.settings.icon_path ? (
                        <img
                          src={convertFileSrc(inst.settings.icon_path)}
                          alt={inst.name}
                          className="w-full h-full object-cover"
                        />
                      ) : (
                        <ServerIcon size={16} />
                      )}
                    </div>
                  </div>
                )}

                {settings.display_server_status && (
                  <div className="relative">
                    <motion.div
                      className={cn(
                        "w-2.5 h-2.5 rounded-full shadow-sm",
                        inst.status === 'Running' ? "bg-accent-emerald" :
                          inst.status === 'Starting' ? "bg-accent-amber" :
                            (inst.status === 'Stopping' || inst.status === 'Crashed') ? "bg-accent-rose" : "bg-gray-400 dark:bg-gray-600"
                      )}
                      animate={(inst.status === 'Running' || inst.status === 'Starting' || inst.status === 'Stopping') ? {
                        scale: [1, 1.2, 1],
                        opacity: [1, 0.7, 1],
                      } : {}}
                      transition={{
                        duration: 2,
                        repeat: Infinity,
                        ease: "easeInOut"
                      }}
                    />
                  </div>
                )}

                <div className="flex flex-col min-w-0">
                  <span className="font-medium truncate leading-none mb-1">{inst.name}</span>
                  <div className="flex items-center gap-1.5">
                    {settings.display_server_version && (
                      <span className={cn(
                        "text-[10px] uppercase font-bold tracking-wider opacity-60 truncate max-w-[80px]",
                        selectedInstanceId === inst.id ? "text-white" : "text-gray-500"
                      )}>
                        {inst.version}
                      </span>
                    )}
                    {settings.display_server_version && settings.display_online_player_count && (
                      <span className={cn(
                        "text-[10px] opacity-40",
                        selectedInstanceId === inst.id ? "text-white" : "text-gray-500"
                      )}>•</span>
                    )}
                    {settings.display_online_player_count && (
                      <span className={cn(
                        "text-[10px] font-bold opacity-60",
                        selectedInstanceId === inst.id ? "text-white" : "text-gray-500"
                      )}>
                        0/{inst.max_players}
                      </span>
                    )}
                  </div>
                </div>

                {settings.display_navigational_buttons && (
                  <div className="ml-auto flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <InstanceSettingsDropdown
                      instance={inst}
                      onUpdated={onInstancesUpdated}
                      size={12}
                      side="left"
                      className={cn(
                        "p-1.5 rounded-lg transition-colors border-0",
                        selectedInstanceId === inst.id ? "hover:bg-white/20" : "hover:bg-black/10 dark:hover:bg-white/10"
                      )}
                    />
                  </div>
                )}
              </motion.button>
            ))}
          </div>
        </div>

        <motion.button
          whileHover={{ scale: 1.02, backgroundColor: "rgba(var(--primary-rgb), 0.05)" }}
          whileTap={{ scale: 0.98 }}
          onClick={onCreateNew}
          className="w-full group flex items-center gap-3 px-4 py-3 rounded-xl border border-dashed border-gray-200 dark:border-white/10 hover:border-primary/50 transition-all duration-300 text-gray-500 dark:text-gray-400 hover:text-primary"
        >
          <div className="p-1.5 bg-gray-100 dark:bg-white/5 rounded-lg group-hover:bg-primary/20 transition-colors">
            <Plus size={18} />
          </div>
          <span className="font-medium">Create New Instance</span>
        </motion.button>
      </div>

      <div className="p-6 border-t border-gray-100 dark:border-white/5 bg-gray-50/50 dark:bg-black/20 transition-colors duration-300">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-primary to-accent-indigo flex items-center justify-center text-white font-bold shadow-lg">
              AD
            </div>
            <div className="flex flex-col min-w-0">
              <span className="text-sm font-semibold text-gray-900 dark:text-white truncate">Administrator</span>
              <span className="text-[10px] text-gray-500 truncate">System Control</span>
            </div>
          </div>
          <motion.button
            whileHover={{ scale: 1.1, rotate: 15 }}
            whileTap={{ scale: 0.9 }}
            onClick={onOpenSettings}
            className="p-2 hover:bg-gray-200 dark:hover:bg-white/5 rounded-lg transition-colors text-gray-400 hover:text-gray-900 dark:hover:text-white group"
            title="App Settings"
          >
            <Settings size={18} className="group-hover:rotate-45 transition-transform duration-300" />
          </motion.button>
        </div>
        <div className="flex items-center justify-between text-[10px] text-gray-400 dark:text-gray-600 font-mono transition-colors duration-300">
          <span>v{appVersion}</span>
          <span className="text-accent-emerald">CONNECTED</span>
        </div>
      </div>
    </div>
  )
}
