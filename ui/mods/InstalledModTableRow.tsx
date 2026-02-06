import { motion } from 'framer-motion'
import { Layers, CheckSquare, Square, ArrowUpCircle, Settings, Trash2, Power, User } from 'lucide-react'
import { InstalledMod, ModUpdate } from '../types'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface InstalledModTableRowProps {
  mod: InstalledMod;
  isSelected: boolean;
  update?: ModUpdate;
  isUpdating: boolean;
  onUpdate: (update: ModUpdate) => void;
  onToggleSelect: () => void;
  onToggleEnabled: () => void;
  onDelete: (delConfig: boolean) => void;
  onConfigure: () => void;
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

export function InstalledModTableRow({
  mod,
  isSelected,
  update,
  isUpdating,
  onUpdate,
  onToggleSelect,
  onToggleEnabled,
  onDelete,
  onConfigure
}: InstalledModTableRowProps) {
  return (
    <motion.tr
      layout
      variants={itemVariants}
      key={mod.filename}
      className={`border-b border-white/5 hover:bg-white/[0.02] transition-colors ${isSelected ? 'bg-primary/5' : ''}`}
    >
      <td className="p-4">
        <button
          onClick={onToggleSelect}
          className={`p-1 rounded-md transition-colors ${isSelected ? 'text-primary' : 'text-gray-600 hover:text-gray-400'}`}
        >
          {isSelected ? <CheckSquare size={18} /> : <Square size={18} />}
        </button>
      </td>
      <td className="p-4 min-w-0 w-full max-w-0">
        <div className="flex items-center gap-3 min-w-0">
          <div className="w-10 h-10 bg-black/20 rounded-lg overflow-hidden flex items-center justify-center shrink-0 border border-white/5">
            {mod.icon_data ? (
              <img src={`data:image/png;base64,${mod.icon_data}`} alt={mod.name} className="w-full h-full object-cover" />
            ) : (
              <Layers size={20} className="text-gray-600" />
            )}
          </div>
          <div className="min-w-0 flex-1 overflow-hidden">
            <div className="font-bold text-gray-200 flex items-center gap-2 truncate">
              <span className="truncate">{mod.name}</span>
              {update && (
                <span className="shrink-0 text-[10px] bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded uppercase tracking-tighter flex items-center gap-1">
                  <ArrowUpCircle size={10} /> Update Available
                </span>
              )}
            </div>
            <div className="text-xs text-gray-500 truncate" title={mod.filename}>{mod.filename}</div>
          </div>
        </div>
      </td>
      <td className="p-4 hidden sm:table-cell">
        <div className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium ${mod.enabled
          ? 'bg-green-500/10 text-green-500'
          : 'bg-red-500/10 text-red-500'
          }`}>
          <Power size={14} />
          {mod.enabled ? 'Enabled' : 'Disabled'}
        </div>
      </td>
      <td className="p-4 text-sm text-gray-400 hidden md:table-cell max-w-[200px]">
        {mod.author ? (
          <div className="flex items-start gap-1.5" title={mod.author}>
            <User size={14} className="opacity-50 shrink-0 mt-0.5" />
            <span className="line-clamp-2 leading-tight">{mod.author}</span>
          </div>
        ) : '—'}
      </td>
      <td className="p-4 text-sm text-gray-400 hidden lg:table-cell truncate">
        <div className="flex flex-col truncate">
          <span className="truncate">{mod.version || 'Unknown'}</span>
          {update && (
            <span className="text-[10px] text-blue-400 font-medium truncate">
              → {update.latest_version}
            </span>
          )}
        </div>
      </td>
      <td className="p-4 text-right shrink-0">
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
            onClick={onToggleEnabled}
            className={`p-2 rounded-lg transition-colors ${mod.enabled
              ? 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20'
              : 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
              }`}
            title={mod.enabled ? 'Disable' : 'Enable'}
          >
            <Power size={16} />
          </button>
          <button
            onClick={onConfigure}
            className="p-2 bg-blue-500/10 text-blue-500 hover:bg-blue-500/20 rounded-lg transition-colors"
            title="Configure Mod"
          >
            <Settings size={16} />
          </button>
          <ConfirmDropdown
            onConfirm={() => onDelete(false)}
            title="Uninstall Mod?"
            message={`Are you sure you want to uninstall ${mod.name}?`}
            confirmText="Uninstall"
            variant="danger"
            className="right-0"
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
