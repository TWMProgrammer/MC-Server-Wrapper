import { motion } from 'framer-motion'
import { FileText } from 'lucide-react'
import { cn } from '../utils'
import { ConfigFile } from './types'

interface ConfigSidebarProps {
  availableConfigs: ConfigFile[]
  selectedConfig: ConfigFile | null
  setSelectedConfig: (config: ConfigFile) => void
}

export function ConfigSidebar({ availableConfigs, selectedConfig, setSelectedConfig }: ConfigSidebarProps) {
  return (
    <div className="w-64 shrink-0 flex flex-col gap-4">
      <div className="flex items-center gap-2 px-2">
        <FileText size={16} className="text-primary" />
        <h3 className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30">
          Config Files
        </h3>
      </div>
      <div className="space-y-1">
        {availableConfigs.map((config) => (
          <button
            key={config.path}
            onClick={() => setSelectedConfig(config)}
            className={cn(
              "w-full text-left px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 flex items-center justify-between group",
              selectedConfig?.path === config.path
                ? "bg-primary text-white shadow-glow-primary"
                : "hover:bg-black/5 dark:hover:bg-white/5 text-gray-500 dark:text-white/40 hover:text-gray-900 dark:hover:text-white"
            )}
          >
            <span className="truncate">{config.name}</span>
            {selectedConfig?.path === config.path && (
              <motion.div layoutId="active-indicator" className="w-1.5 h-1.5 rounded-full bg-white" />
            )}
          </button>
        ))}
      </div>
    </div>
  )
}
