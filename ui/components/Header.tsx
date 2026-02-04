import { Database, Network, Beaker, Users, Tag, Play, Square, Settings2, FolderOpen, Loader2, RotateCcw } from 'lucide-react'
import { motion } from 'framer-motion'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Instance, TabId, TransitionType } from '../types'
import { cn } from '../utils'
import { InstanceFolderDropdown } from '../InstanceFolderDropdown'
import { InstanceSettingsDropdown } from '../InstanceSettingsDropdown'

interface HeaderProps {
  currentInstance: Instance;
  status: string;
  isTransitioning?: TransitionType | null;
  activeTab: TabId;
  tabs: { id: TabId; label: string; icon: any }[];
  onStartServer: () => void;
  onStopServer: () => void;
  onRestartServer: () => void;
  onSetActiveTab: (tab: TabId) => void;
  onInstancesUpdated: () => void;
}

export function Header({
  currentInstance,
  status,
  isTransitioning = null,
  activeTab,
  tabs,
  onStartServer,
  onStopServer,
  onRestartServer,
  onSetActiveTab,
  onInstancesUpdated
}: HeaderProps) {
  return (
    <div className="px-8 pt-8 pb-0 bg-surface/50 border-b border-gray-100 dark:border-white/5 backdrop-blur-xl sticky top-0 z-20 transition-colors duration-300">
      <div className="flex items-start justify-between mb-8">
        <div className="flex items-start gap-6">
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="w-16 h-16 bg-gradient-to-br from-primary to-accent-indigo rounded-2xl flex items-center justify-center shadow-2xl shadow-primary/20 ring-1 ring-white/20 overflow-hidden"
          >
            {currentInstance.settings.icon_path ? (
              <img
                src={convertFileSrc(currentInstance.settings.icon_path)}
                alt={currentInstance.name}
                className="w-full h-full object-cover"
              />
            ) : (
              <Database size={32} className="text-white" />
            )}
          </motion.div>

          <div className="space-y-2 min-w-0 flex-1">
            <div className="flex items-center gap-4">
              <h2 className="text-3xl font-black tracking-tight text-gray-900 dark:text-white transition-colors duration-300 truncate">{currentInstance.name}</h2>
              <div className={cn(
                "flex items-center gap-2 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest ring-1 transition-all duration-300",
                (status === 'Running' || isTransitioning === 'starting' || (isTransitioning === 'restarting' && status === 'Starting')) ? "bg-accent-emerald/10 text-accent-emerald ring-accent-emerald/20" :
                  (status === 'Starting' || (isTransitioning as any) === 'starting') ? "bg-accent-amber/10 text-accent-amber ring-accent-amber/20" :
                    (status === 'Stopping' || isTransitioning === 'stopping' || status === 'Crashed') ? "bg-accent-rose/10 text-accent-rose ring-accent-rose/20" : "bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-400 ring-gray-200 dark:ring-gray-700"
              )}>
                <motion.div
                  className={cn(
                    "w-1.5 h-1.5 rounded-full shadow-sm",
                    (status === 'Running' || isTransitioning === 'starting' || (isTransitioning === 'restarting' && status === 'Starting')) ? "bg-accent-emerald" :
                      (status === 'Starting' || (isTransitioning as any) === 'starting') ? "bg-accent-amber" :
                        (status === 'Stopping' || isTransitioning === 'stopping' || status === 'Crashed') ? "bg-accent-rose" : "bg-gray-400 dark:bg-gray-500"
                  )}
                  animate={(status === 'Running' || status === 'Starting' || isTransitioning) ? {
                    scale: [1, 1.2, 1],
                    opacity: [1, 0.7, 1],
                  } : {}}
                  transition={{
                    duration: 2,
                    repeat: Infinity,
                    ease: "easeInOut"
                  }}
                />
                {isTransitioning === 'starting' ? 'Starting...' :
                  isTransitioning === 'stopping' ? 'Stopping...' :
                    isTransitioning === 'restarting' ? 'Restarting...' :
                      status === 'Stopped' ? 'Offline' : status}
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-gray-500 dark:text-gray-400 transition-colors duration-300">
              <div className="flex items-center gap-2 text-sm font-medium hover:text-gray-900 dark:hover:text-gray-200 transition-colors cursor-default">
                <Network size={16} className="text-primary/60" />
                <span>{currentInstance.ip}:{currentInstance.port}</span>
              </div>
              <div className="flex items-center gap-2 text-sm font-medium hover:text-gray-900 dark:hover:text-gray-200 transition-colors cursor-default">
                <Beaker size={16} className="text-primary/60" />
                <span>{currentInstance.server_type} {currentInstance.version}</span>
              </div>
              <div className="flex items-center gap-2 text-sm font-medium hover:text-gray-900 dark:hover:text-gray-200 transition-colors cursor-default">
                <Users size={16} className="text-primary/60" />
                <span>0/{currentInstance.max_players} players</span>
              </div>
              {currentInstance.description && (
                <div className="flex items-center gap-2 text-sm font-medium hover:text-gray-900 dark:hover:text-gray-200 transition-colors cursor-default">
                  <Tag size={16} className="text-primary/60" />
                  <span className="max-w-xs truncate">{currentInstance.description}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 mr-4">
            <InstanceFolderDropdown instance={currentInstance} />
            <InstanceSettingsDropdown
              instance={currentInstance}
              onUpdated={onInstancesUpdated}
            />
          </div>

          {status === 'Stopped' || status === 'Crashed' || status === 'Starting' || isTransitioning === 'starting' ? (
            <motion.button
              whileHover={isTransitioning ? {} : { scale: 1.05 }}
              whileTap={isTransitioning ? {} : { scale: 0.95 }}
              onClick={onStartServer}
              disabled={!!isTransitioning || status === 'Starting'}
              className={cn(
                "flex items-center gap-2 px-8 py-3 bg-primary hover:bg-primary-hover text-white rounded-xl font-bold transition-all shadow-glow-primary shadow-primary/20",
                (isTransitioning || status === 'Starting') && "opacity-80 cursor-not-allowed"
              )}
            >
              {isTransitioning === 'starting' || status === 'Starting' ? (
                <Loader2 size={20} className="animate-spin" />
              ) : (
                <Play size={20} fill="currentColor" />
              )}
              {isTransitioning === 'starting' || status === 'Starting' ? 'Starting...' : 'Start Server'}
            </motion.button>
          ) : (
            <motion.button
              whileHover={isTransitioning ? {} : { scale: 1.05 }}
              whileTap={isTransitioning ? {} : { scale: 0.95 }}
              onClick={onStopServer}
              disabled={!!isTransitioning || status === 'Stopping'}
              className={cn(
                "flex items-center gap-2 px-8 py-3 bg-accent-rose/10 hover:bg-accent-rose/20 text-accent-rose rounded-xl font-bold transition-all ring-1 ring-accent-rose/30",
                (isTransitioning || status === 'Stopping') && "opacity-80 cursor-not-allowed"
              )}
            >
              {isTransitioning === 'stopping' || status === 'Stopping' ? (
                <Loader2 size={20} className="animate-spin" />
              ) : (
                <Square size={20} fill="currentColor" />
              )}
              {isTransitioning === 'stopping' || status === 'Stopping' ? 'Stopping...' :
                isTransitioning === 'restarting' ? 'Restarting...' : 'Stop Server'}
            </motion.button>
          )}

          {(status === 'Running' || status === 'Starting' || status === 'Stopping') && (
            <motion.button
              whileHover={isTransitioning ? {} : { scale: 1.05 }}
              whileTap={isTransitioning ? {} : { scale: 0.95 }}
              onClick={onRestartServer}
              disabled={!!isTransitioning}
              className={cn(
                "flex items-center justify-center w-12 h-12 bg-accent-amber/10 hover:bg-accent-amber/20 text-accent-amber rounded-xl font-bold transition-all ring-1 ring-accent-amber/30",
                isTransitioning && "opacity-80 cursor-not-allowed"
              )}
              title="Restart Server"
            >
              {isTransitioning === 'restarting' ? (
                <Loader2 size={20} className="animate-spin" />
              ) : (
                <RotateCcw size={20} />
              )}
            </motion.button>
          )}
        </div>
      </div>

      <div className="flex gap-8 overflow-x-auto no-scrollbar">
        {tabs.map((tab) => (
          <motion.button
            key={tab.id}
            whileHover={{ y: -2 }}
            whileTap={{ scale: 0.95 }}
            onClick={() => onSetActiveTab(tab.id)}
            className={cn(
              "flex items-center gap-2.5 pb-4 px-1 text-sm font-bold uppercase tracking-widest transition-all relative",
              activeTab === tab.id
                ? "text-primary"
                : "text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
            )}
          >
            <tab.icon size={18} />
            <span>{tab.label}</span>
            {activeTab === tab.id && (
              <motion.div
                layoutId="activeTab"
                className="absolute bottom-0 left-0 right-0 h-1 bg-primary rounded-t-full shadow-glow-primary shadow-primary/40"
              />
            )}
          </motion.button>
        ))}
      </div>
    </div>
  )
}
