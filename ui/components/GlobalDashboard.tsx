import { motion } from 'framer-motion'
import { Database, Play, Square, Network, Beaker, Users, Activity, Loader2 } from 'lucide-react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Instance, TransitionType } from '../types'
import { cn } from '../utils'
import { AppSettings } from '../hooks/useAppSettings'

interface GlobalDashboardProps {
  instances: Instance[];
  isTransitioning: Record<string, TransitionType | null>;
  onSelectInstance: (id: string) => void;
  onStartServer: (id: string) => void;
  onStopServer: (id: string) => void;
  settings: AppSettings;
}

export function GlobalDashboard({
  instances,
  isTransitioning,
  onSelectInstance,
  onStartServer,
  onStopServer,
  settings
}: GlobalDashboardProps) {
  return (
    <div className="space-y-8 animate-fade-in">
      <div className="flex flex-col gap-2">
        <h2 className="text-3xl font-black tracking-tight text-gray-900 dark:text-white transition-colors duration-300">
          Server <span className="text-primary">Overview</span>
        </h2>
        <p className="text-gray-500 dark:text-gray-400 font-medium">
          Manage all your Minecraft server instances from one place.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
        {instances.map((instance, i) => (
          <motion.div
            key={instance.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            whileHover={{ scale: 1.02, y: -4 }}
            whileTap={{ scale: 0.98 }}
            transition={{
              delay: i * 0.05,
              scale: { type: "spring", stiffness: 400, damping: 25 }
            }}
            className="card group transition-all duration-300 flex flex-col h-full border border-black/10 dark:border-white/5"
          >
            <div className="flex items-start justify-between mb-6">
              <div className="flex items-center gap-4">
                <div className={cn(
                  "w-12 h-12 rounded-2xl bg-gradient-to-br transition-all duration-300 shadow-lg group-hover:shadow-primary/20 flex items-center justify-center overflow-hidden",
                  instance.status === 'Running' ? "from-accent-emerald to-emerald-600 text-white" :
                    instance.status === 'Starting' ? "from-accent-amber to-amber-600 text-white" :
                      "from-gray-200 to-gray-300 dark:from-gray-700 dark:to-gray-800 text-gray-500 dark:text-gray-400"
                )}>
                  {instance.settings.icon_path ? (
                    <img
                      src={convertFileSrc(instance.settings.icon_path)}
                      alt={instance.name}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <Database size={24} />
                  )}
                </div>
                <div>
                  <h3 className="text-xl font-black text-gray-900 dark:text-white truncate max-w-[150px]">
                    {instance.name}
                  </h3>
                  <div className="flex items-center gap-2">
                    <motion.div
                      className={cn(
                        "w-2 h-2 rounded-full",
                        (instance.status === 'Running' || isTransitioning[instance.id] === 'starting' || (isTransitioning[instance.id] === 'restarting' && instance.status === 'Starting')) ? "bg-accent-emerald" :
                          (instance.status === 'Starting' || (isTransitioning[instance.id] as any) === 'starting') ? "bg-accent-amber" :
                            (instance.status === 'Stopping' || isTransitioning[instance.id] === 'stopping') ? "bg-accent-rose" :
                              "bg-gray-400 dark:bg-gray-500"
                      )}
                      animate={(instance.status === 'Running' || instance.status === 'Starting' || isTransitioning[instance.id]) ? {
                        scale: [1, 1.2, 1],
                        opacity: [1, 0.7, 1],
                      } : {}}
                      transition={{
                        duration: 2,
                        repeat: Infinity,
                        ease: "easeInOut"
                      }}
                    />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-gray-500">
                      {isTransitioning[instance.id] === 'starting' ? 'Starting...' :
                        isTransitioning[instance.id] === 'stopping' ? 'Stopping...' :
                          isTransitioning[instance.id] === 'restarting' ? 'Restarting...' :
                            instance.status === 'Stopped' ? 'Offline' : instance.status}
                    </span>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-2">
                {instance.status === 'Stopped' || instance.status === 'Crashed' || instance.status === 'Starting' || isTransitioning[instance.id] === 'starting' ? (
                  <motion.button
                    whileHover={isTransitioning[instance.id] || instance.status === 'Starting' ? {} : { scale: 1.1 }}
                    whileTap={isTransitioning[instance.id] || instance.status === 'Starting' ? {} : { scale: 0.9 }}
                    disabled={!!isTransitioning[instance.id] || instance.status === 'Starting'}
                    onClick={(e) => {
                      e.stopPropagation();
                      onStartServer(instance.id);
                    }}
                    className={cn(
                      "p-2 rounded-lg bg-accent-emerald/10 text-accent-emerald hover:bg-accent-emerald hover:text-white transition-all duration-300",
                      (isTransitioning[instance.id] || instance.status === 'Starting') && "opacity-50 cursor-not-allowed"
                    )}
                    title="Start Server"
                  >
                    {isTransitioning[instance.id] === 'starting' || instance.status === 'Starting' ? (
                      <Loader2 size={18} className="animate-spin" />
                    ) : (
                      <Play size={18} fill="currentColor" />
                    )}
                  </motion.button>
                ) : (
                  <motion.button
                    whileHover={isTransitioning[instance.id] || instance.status === 'Stopping' ? {} : { scale: 1.1 }}
                    whileTap={isTransitioning[instance.id] || instance.status === 'Stopping' ? {} : { scale: 0.9 }}
                    disabled={!!isTransitioning[instance.id] || instance.status === 'Stopping'}
                    onClick={(e) => {
                      e.stopPropagation();
                      onStopServer(instance.id);
                    }}
                    className={cn(
                      "p-2 rounded-lg bg-accent-rose/10 text-accent-rose hover:bg-accent-rose hover:text-white transition-all duration-300",
                      (isTransitioning[instance.id] || instance.status === 'Stopping') && "opacity-50 cursor-not-allowed"
                    )}
                    title="Stop Server"
                  >
                    {isTransitioning[instance.id] === 'stopping' || instance.status === 'Stopping' || isTransitioning[instance.id] === 'restarting' ? (
                      <Loader2 size={18} className="animate-spin" />
                    ) : (
                      <Square size={18} fill="currentColor" />
                    )}
                  </motion.button>
                )}
              </div>
            </div>

            <div className="space-y-3 flex-1">
              <div className="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400 font-medium">
                <Network size={16} className="text-primary/60" />
                <span>
                  {settings.hide_ip_address ? (
                    <span className="flex items-center gap-1">
                      <span className="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
                      <span className="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
                      <span className="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
                      <span className="ml-1 text-[10px] uppercase font-bold tracking-widest opacity-40">Hidden</span>
                    </span>
                  ) : (
                    <>
                      {settings.display_ipv6 ? (
                        instance.ip === '127.0.0.1' || instance.ip === 'localhost' ? '::1' : `::ffff:${instance.ip}`
                      ) : instance.ip}:{instance.port}
                    </>
                  )}
                </span>
              </div>
              <div className="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400 font-medium">
                <Beaker size={16} className="text-primary/60" />
                <span>{instance.server_type} {instance.version}</span>
              </div>
              <div className="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400 font-medium">
                <Users size={16} className="text-primary/60" />
                <span>0/{instance.max_players} players</span>
              </div>
            </div>

            <div className="mt-6 pt-6 border-t border-black/5 dark:border-white/5">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={() => onSelectInstance(instance.id)}
                className="w-full py-3 rounded-xl bg-primary/10 text-primary font-bold hover:bg-primary hover:text-white transition-all duration-300 flex items-center justify-center gap-2"
              >
                <Activity size={18} />
                Manage Instance
              </motion.button>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  )
}
