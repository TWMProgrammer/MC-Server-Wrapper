import { motion } from 'framer-motion'
import { Package, Power, Settings, Trash2, ArrowUpCircle, User, CheckSquare, Square } from 'lucide-react'
import { InstalledPlugin, PluginUpdate } from '../types'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface PluginTableRowProps {
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
  hidden: { opacity: 0, y: 10 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      type: "spring" as const,
      stiffness: 300,
      damping: 25
    }
  },
  exit: {
    opacity: 0,
    scale: 0.95,
    transition: { duration: 0.2 }
  }
};

export function PluginTableRow({
  plugin,
  isSelected,
  update,
  isUpdating,
  onToggleSelection,
  onToggle,
  onUpdate,
  onOpenConfig,
  onDelete
}: PluginTableRowProps) {
  return (
    <motion.tr
      layout
      variants={itemVariants}
      key={plugin.filename}
      className={`hover:bg-white/5 transition-colors ${!plugin.enabled ? 'opacity-60 grayscale-[0.5]' : ''} ${isSelected ? 'bg-primary/5' : ''}`}
    >
      <td className="px-6 py-4">
        <button
          onClick={() => onToggleSelection(plugin.filename)}
          className={`p-1 rounded transition-colors ${isSelected
            ? 'text-primary'
            : 'text-gray-600 hover:text-gray-400'
            }`}
        >
          {isSelected ? <CheckSquare size={18} /> : <Square size={18} />}
        </button>
      </td>
      <td className="px-6 py-4 min-w-0 w-full max-w-0">
        <div className="flex items-center gap-3 min-w-0">
          <div className={`p-2 rounded-lg shrink-0 ${plugin.enabled ? 'bg-primary/10 text-primary' : 'bg-gray-500/10 text-gray-500'}`}>
            <Package size={20} />
          </div>
          <div className="min-w-0 flex-1 overflow-hidden">
            <div className="font-medium flex items-center gap-2 truncate">
              <span className="truncate">{plugin.name}</span>
              {!plugin.enabled && (
                <span className="shrink-0 text-[10px] bg-gray-500/20 text-gray-400 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                  Disabled
                </span>
              )}
              {update && (
                <span className="shrink-0 text-[10px] bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded uppercase tracking-tighter flex items-center gap-1">
                  <ArrowUpCircle size={10} /> Update Available
                </span>
              )}
            </div>
            <div className="text-xs text-gray-500 truncate" title={plugin.description || plugin.filename}>
              {plugin.description || plugin.filename}
            </div>
          </div>
        </div>
      </td>
      <td className="px-6 py-4 hidden sm:table-cell">
        <div className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium ${plugin.enabled
          ? 'bg-green-500/10 text-green-500'
          : 'bg-red-500/10 text-red-500'
          }`}>
          <Power size={14} />
          {plugin.enabled ? 'Enabled' : 'Disabled'}
        </div>
      </td>
      <td className="px-6 py-4 text-sm text-gray-400 hidden md:table-cell max-w-[200px]">
        {plugin.author ? (
          <div className="flex items-start gap-1.5" title={plugin.author}>
            <User size={14} className="opacity-50 shrink-0 mt-0.5" />
            <span className="line-clamp-2 leading-tight">{plugin.author}</span>
          </div>
        ) : '—'}
      </td>
      <td className="px-6 py-4 text-sm text-gray-400 hidden lg:table-cell truncate">
        <div className="flex flex-col truncate">
          <span className="truncate">{plugin.version || 'Unknown'}</span>
          {update && (
            <span className="text-[10px] text-blue-400 font-medium truncate">
              → {update.latest_version}
            </span>
          )}
        </div>
      </td>
      <td className="px-6 py-4 text-right shrink-0">
        <div className="flex items-center justify-end gap-2 shrink-0">
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
      </td>
    </motion.tr>
  )
}
