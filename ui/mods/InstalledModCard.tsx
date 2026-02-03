import { motion } from 'framer-motion'
import { Layers, CheckSquare, Square, Power, ArrowUpCircle, User, Sliders, Trash2 } from 'lucide-react'
import { InstalledMod, ModUpdate } from '../types'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface InstalledModCardProps {
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

export function InstalledModCard({
  mod,
  isSelected,
  update,
  isUpdating,
  onUpdate,
  onToggleSelect,
  onToggleEnabled,
  onDelete,
  onConfigure
}: InstalledModCardProps) {
  return (
    <motion.div
      layout
      className={`group relative p-4 bg-white/5 rounded-2xl border transition-all ${isSelected ? 'border-primary/50 bg-primary/5 shadow-lg shadow-primary/5' : 'border-white/5 hover:border-white/10 hover:bg-white/10'}`}
    >
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex items-center gap-3">
          <div className="relative w-12 h-12 bg-black/20 rounded-xl overflow-hidden flex items-center justify-center flex-shrink-0 border border-white/5">
            {mod.icon_data ? (
              <img src={`data:image/png;base64,${mod.icon_data}`} alt={mod.name} className="w-full h-full object-cover" />
            ) : (
              <Layers size={24} className="text-gray-600" />
            )}
            <button
              onClick={(e) => { e.stopPropagation(); onToggleSelect(); }}
              className={`absolute top-0 left-0 w-full h-full flex items-center justify-center bg-primary/80 text-white transition-opacity ${isSelected ? 'opacity-100' : 'opacity-0 group-hover:opacity-0'}`}
            >
              <CheckSquare size={20} />
            </button>
            {!isSelected && (
              <button
                onClick={(e) => { e.stopPropagation(); onToggleSelect(); }}
                className="absolute top-0 left-0 w-full h-full flex items-center justify-center bg-black/40 text-white opacity-0 group-hover:opacity-100 transition-opacity"
              >
                <Square size={20} />
              </button>
            )}
          </div>
          <div className="min-w-0">
            <h4 className="font-bold truncate text-gray-200 flex items-center gap-2">
              {mod.name}
              {update && (
                <span className="flex-shrink-0 w-2 h-2 bg-blue-500 rounded-full animate-pulse" title={`Update available: ${update.latest_version}`} />
              )}
            </h4>
            <div className="flex items-center gap-2 mt-0.5">
              <span className="text-[10px] font-black uppercase px-1.5 py-0.5 bg-white/5 rounded text-gray-500">
                {mod.version || 'v?.?.?'}
              </span>
              {mod.loader && (
                <span className="text-[10px] font-black uppercase px-1.5 py-0.5 bg-primary/10 text-primary rounded">
                  {mod.loader}
                </span>
              )}
            </div>
          </div>
        </div>
        <button
          onClick={onToggleEnabled}
          className={`p-2 rounded-xl transition-all ${mod.enabled ? 'bg-green-500/10 text-green-500 hover:bg-green-500/20' : 'bg-red-500/10 text-red-500 hover:bg-red-500/20'}`}
          title={mod.enabled ? 'Enabled' : 'Disabled'}
        >
          <Power size={18} className={mod.enabled ? '' : 'rotate-180'} />
        </button>
      </div>

      <p className="text-sm text-gray-500 line-clamp-2 mb-4 h-10">
        {mod.description || 'No description available.'}
      </p>

      {update && (
        <div className="mb-4 p-2 bg-blue-500/10 border border-blue-500/20 rounded-xl flex items-center justify-between">
          <div className="text-[10px] text-blue-400 font-bold uppercase flex items-center gap-1">
            <ArrowUpCircle size={12} /> Update to {update.latest_version}
          </div>
          <button
            onClick={() => onUpdate(update)}
            disabled={isUpdating}
            className="px-2 py-1 bg-blue-500 text-white text-[10px] font-bold rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
          >
            {isUpdating ? 'Updating...' : 'Update'}
          </button>
        </div>
      )}

      <div className="flex items-center justify-between mt-auto pt-4 border-t border-white/5">
        <div className="flex items-center gap-2 text-xs text-gray-500">
          <User size={14} />
          <span className="truncate max-w-[100px]">{mod.author || 'Unknown'}</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={onConfigure}
            className="p-2 text-gray-500 hover:text-primary hover:bg-primary/10 rounded-lg transition-all"
            title="Configure Mod"
          >
            <Sliders size={16} />
          </button>
          <ConfirmDropdown
            onConfirm={() => onDelete(false)}
            title="Uninstall Mod?"
            message={`Are you sure you want to uninstall ${mod.name}?`}
            confirmText="Uninstall"
            className="right-0 bottom-full mb-2"
          >
            <button className="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all">
              <Trash2 size={16} />
            </button>
          </ConfirmDropdown>
        </div>
      </div>
    </motion.div>
  )
}
