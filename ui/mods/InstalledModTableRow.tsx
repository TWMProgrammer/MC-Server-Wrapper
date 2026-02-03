import { Layers, CheckSquare, Square, ArrowUpCircle, Sliders, Trash2 } from 'lucide-react'
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
    <tr className={`border-b border-white/5 hover:bg-white/[0.02] transition-colors ${isSelected ? 'bg-primary/5' : ''}`}>
      <td className="p-4">
        <button
          onClick={onToggleSelect}
          className={`p-1 rounded-md transition-colors ${isSelected ? 'text-primary' : 'text-gray-600 hover:text-gray-400'}`}
        >
          {isSelected ? <CheckSquare size={18} /> : <Square size={18} />}
        </button>
      </td>
      <td className="p-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-black/20 rounded-lg overflow-hidden flex items-center justify-center flex-shrink-0 border border-white/5">
            {mod.icon_data ? (
              <img src={`data:image/png;base64,${mod.icon_data}`} alt={mod.name} className="w-full h-full object-cover" />
            ) : (
              <Layers size={20} className="text-gray-600" />
            )}
          </div>
          <div>
            <div className="font-bold text-gray-200 flex items-center gap-2">
              {mod.name}
              {update && (
                <span className="text-[10px] bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded uppercase tracking-tighter flex items-center gap-1">
                  <ArrowUpCircle size={10} /> Update Available
                </span>
              )}
            </div>
            <div className="text-xs text-gray-500 truncate max-w-[200px]">{mod.filename}</div>
          </div>
        </div>
      </td>
      <td className="p-4 text-sm text-gray-400">
        <div className="flex flex-col">
          <span>{mod.version || 'Unknown'}</span>
          {update && (
            <span className="text-[10px] text-blue-400 font-medium">
              → {update.latest_version}
            </span>
          )}
        </div>
      </td>
      <td className="p-4">
        {mod.loader ? (
          <span className="text-[10px] font-black uppercase px-2 py-1 bg-primary/10 text-primary rounded-full">
            {mod.loader}
          </span>
        ) : (
          <span className="text-[10px] font-black uppercase px-2 py-1 bg-white/5 text-gray-500 rounded-full">
            Universal
          </span>
        )}
      </td>
      <td className="p-4">
        <button
          onClick={onToggleEnabled}
          className={`flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold transition-all ${mod.enabled ? 'bg-green-500/10 text-green-500' : 'bg-red-500/10 text-red-500'}`}
        >
          <div className={`w-1.5 h-1.5 rounded-full ${mod.enabled ? 'bg-green-500' : 'bg-red-500'}`} />
          {mod.enabled ? 'Enabled' : 'Disabled'}
        </button>
      </td>
      <td className="p-4 text-right">
        <div className="flex items-center justify-end gap-1">
          {update && (
            <button
              onClick={() => onUpdate(update)}
              disabled={isUpdating}
              className="p-2 text-blue-400 hover:bg-blue-400/10 rounded-lg transition-all"
              title={`Update to ${update.latest_version}`}
            >
              <ArrowUpCircle size={16} className={isUpdating ? 'animate-spin' : ''} />
            </button>
          )}
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
            className="right-0"
          >
            <button className="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all">
              <Trash2 size={16} />
            </button>
          </ConfirmDropdown>
        </div>
      </td>
    </tr>
  )
}
