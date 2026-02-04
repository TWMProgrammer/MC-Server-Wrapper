import { motion } from 'framer-motion'
import { Package, Power, Settings, Trash2, ArrowUpCircle, Info, User, CheckSquare, Square } from 'lucide-react'
import { InstalledPlugin, PluginUpdate } from '../types'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface PluginCardProps {
  plugin: InstalledPlugin;
  isSelected: boolean;
  update?: PluginUpdate;
  isUpdating: boolean;
  onToggleSelection: (filename: string) => void;
  onToggle: (plugin: InstalledPlugin) => void;
  onUpdate: (update: PluginUpdate) => void;
  onOpenConfig: (plugin: InstalledPlugin) => void;
  onDelete: (plugin: InstalledPlugin, deleteConfig: boolean) => void;
}

const itemVariants = {
  hidden: { opacity: 0, scale: 0.9, y: 10 },
  visible: {
    opacity: 1,
    scale: 1,
    y: 0,
    transition: {
      type: "spring" as const,
      stiffness: 300,
      damping: 25
    }
  },
  exit: {
    opacity: 0,
    scale: 0.9,
    transition: { duration: 0.2 }
  }
};

export function PluginCard({
  plugin,
  isSelected,
  update,
  isUpdating,
  onToggleSelection,
  onToggle,
  onUpdate,
  onOpenConfig,
  onDelete
}: PluginCardProps) {
  return (
    <motion.div
      layout
      variants={itemVariants}
      key={plugin.filename}
      className={`bg-white/5 border rounded-2xl p-4 hover:border-primary/30 transition-all group flex flex-col h-full relative ${!plugin.enabled ? 'opacity-60 grayscale-[0.5]' : ''} ${isSelected ? 'border-primary/50 bg-primary/5 shadow-lg shadow-primary/5' : 'border-white/5'}`}
    >
      <button
        onClick={() => onToggleSelection(plugin.filename)}
        className={`absolute top-4 right-4 p-1.5 rounded-lg transition-all z-10 ${isSelected
          ? 'bg-primary text-white shadow-lg shadow-primary/20 opacity-100'
          : 'bg-black/40 text-gray-500 opacity-0 group-hover:opacity-100 hover:text-white'
          }`}
      >
        {isSelected ? <CheckSquare size={16} /> : <Square size={16} />}
      </button>

      <div className="flex items-start justify-between mb-3">
        <div className={`p-3 rounded-xl ${plugin.enabled ? 'bg-primary/10 text-primary' : 'bg-gray-500/10 text-gray-500'}`}>
          <Package size={24} />
        </div>
        <div className="flex items-center gap-1 mr-8">
          {update && (
            <button
              onClick={() => onUpdate(update)}
              disabled={isUpdating}
              className="p-2 bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 rounded-lg transition-colors"
              title={`Update to ${update.latest_version}`}
            >
              <ArrowUpCircle size={16} className={isUpdating ? 'animate-spin' : ''} />
            </button>
          )}
          <button
            onClick={() => onToggle(plugin)}
            className={`p-2 rounded-lg transition-colors ${plugin.enabled
              ? 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20'
              : 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
              }`}
            title={plugin.enabled ? 'Disable' : 'Enable'}
          >
            <Power size={16} />
          </button>
          <button
            onClick={() => onOpenConfig(plugin)}
            className="p-2 bg-blue-500/10 text-blue-500 hover:bg-blue-500/20 rounded-lg transition-colors"
            title="Configure"
          >
            <Settings size={16} />
          </button>
          <ConfirmDropdown
            title="Uninstall Plugin"
            message={`Are you sure you want to remove ${plugin.name}?`}
            confirmText="Uninstall"
            variant="danger"
            onConfirm={() => onDelete(plugin, false)}
          >
            <button className="p-2 bg-red-500/10 text-red-500 hover:bg-red-500/20 rounded-lg transition-colors">
              <Trash2 size={16} />
            </button>
          </ConfirmDropdown>
        </div>
      </div>

      <div className="flex-1">
        <div className="font-bold text-lg flex flex-wrap items-center gap-2 mb-1">
          {plugin.name}
          {!plugin.enabled && (
            <span className="text-[10px] bg-gray-500/20 text-gray-400 px-1.5 py-0.5 rounded uppercase tracking-tighter">
              Disabled
            </span>
          )}
          {update && (
            <span className="text-[10px] bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded uppercase tracking-tighter flex items-center gap-1">
              <ArrowUpCircle size={10} /> Update Available
            </span>
          )}
        </div>
        <p className="text-sm text-gray-500 line-clamp-2 mb-4 h-10">
          {plugin.description || "No description available."}
        </p>
      </div>

      <div className="flex items-center justify-between mt-4 pt-4 border-t border-white/5 text-xs text-gray-400">
        <div className="flex items-center gap-3">
          <span className="flex flex-col">
            <span className="flex items-center gap-1">
              <Info size={12} className="opacity-50" />
              v{plugin.version || '?.?.?'}
            </span>
            {update && (
              <span className="text-[10px] text-blue-400 font-medium">
                → {update.latest_version}
              </span>
            )}
          </span>
          {plugin.author && (
            <span className="flex items-center gap-1 truncate max-w-[80px]">
              <User size={12} className="opacity-50" />
              {plugin.author}
            </span>
          )}
        </div>
        <div className="font-mono opacity-30 text-[10px] truncate max-w-[80px]">
          {plugin.filename}
        </div>
      </div>
    </motion.div>
  )
}
