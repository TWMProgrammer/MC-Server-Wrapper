import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Layers,
  Trash2,
  RefreshCw,
  Search,
  Power,
  Package,
  Info,
  LayoutGrid,
  List,
  User,
  Settings,
  Sliders,
  CheckSquare,
  Square,
  ArrowUpCircle,
  AlertTriangle,
  ChevronDown
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { InstalledMod } from '../types'
import { useToast } from '../hooks/useToast'
import { ConfirmDropdown } from '../components/ConfirmDropdown'
import { ModConfigModal } from './ModConfigModal'

interface InstalledModsProps {
  instanceId: string;
  refreshTrigger?: number;
}

type ViewMode = 'table' | 'grid'

export function InstalledMods({ instanceId, refreshTrigger }: InstalledModsProps) {
  const [mods, setMods] = useState<InstalledMod[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<ViewMode>('grid')
  const [selectedFilenames, setSelectedFilenames] = useState<Set<string>>(new Set())
  const [configuringMod, setConfiguringMod] = useState<InstalledMod | null>(null)
  const { showToast } = useToast()

  useEffect(() => {
    loadMods()
  }, [instanceId, refreshTrigger])

  const loadMods = async () => {
    setLoading(true)
    try {
      const result = await invoke<InstalledMod[]>('list_installed_mods', { instanceId })
      setMods(result)
      setSelectedFilenames(new Set())
    } catch (err) {
      console.error('Failed to load mods:', err)
      showToast('Failed to load mods', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleToggleMod = async (mod: InstalledMod) => {
    try {
      await invoke('toggle_mod', {
        instanceId,
        filename: mod.filename,
        enable: !mod.enabled
      })
      showToast(`Mod ${!mod.enabled ? 'enabled' : 'disabled'} successfully`)
      await loadMods()
    } catch (err) {
      console.error('Failed to toggle mod:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleDeleteMod = async (mod: InstalledMod, deleteConfig: boolean) => {
    try {
      await invoke('uninstall_mod', {
        instanceId,
        filename: mod.filename,
        deleteConfig
      })
      showToast('Mod uninstalled successfully')
      await loadMods()
    } catch (err) {
      console.error('Failed to uninstall mod:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleBulkToggle = async (enable: boolean) => {
    try {
      await invoke('bulk_toggle_mods', {
        instanceId,
        filenames: Array.from(selectedFilenames),
        enable
      })
      showToast(`Bulk ${enable ? 'enable' : 'disable'} successful`)
      await loadMods()
    } catch (err) {
      showToast(`Bulk toggle failed: ${err}`, 'error')
    }
  }

  const handleBulkDelete = async (deleteConfig: boolean) => {
    try {
      await invoke('bulk_uninstall_mods', {
        instanceId,
        filenames: Array.from(selectedFilenames),
        deleteConfig
      })
      showToast(`Bulk uninstall successful`)
      await loadMods()
    } catch (err) {
      showToast(`Bulk uninstall failed: ${err}`, 'error')
    }
  }

  const toggleSelection = (filename: string) => {
    setSelectedFilenames(prev => {
      const next = new Set(prev)
      if (next.has(filename)) next.delete(filename)
      else next.add(filename)
      return next
    })
  }

  const toggleAll = () => {
    if (selectedFilenames.size === filteredMods.length) {
      setSelectedFilenames(new Set())
    } else {
      setSelectedFilenames(new Set(filteredMods.map(p => p.filename)))
    }
  }

  const filteredMods = mods.filter(m =>
    m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    m.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
    m.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    m.author?.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
            <input
              type="text"
              placeholder="Search installed mods..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-black/20 border border-white/5 rounded-xl focus:outline-none focus:border-primary/50 transition-colors"
            />
          </div>
          <div className="flex items-center bg-white/5 p-1 rounded-xl border border-white/5">
            <button
              onClick={() => setViewMode('table')}
              className={`p-1.5 rounded-lg transition-all ${viewMode === 'table' ? 'bg-primary text-white shadow-lg shadow-primary/20' : 'text-gray-500 hover:text-gray-300'}`}
              title="Table View"
            >
              <List size={18} />
            </button>
            <button
              onClick={() => setViewMode('grid')}
              className={`p-1.5 rounded-lg transition-all ${viewMode === 'grid' ? 'bg-primary text-white shadow-lg shadow-primary/20' : 'text-gray-500 hover:text-gray-300'}`}
              title="Grid View"
            >
              <LayoutGrid size={18} />
            </button>
          </div>
          <button
            onClick={loadMods}
            disabled={loading}
            className="p-2.5 bg-white/5 hover:bg-white/10 text-gray-400 rounded-xl transition-all border border-white/5"
            title="Refresh list"
          >
            <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>

        {selectedFilenames.size > 0 && (
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex items-center gap-2 p-1 bg-primary/10 border border-primary/20 rounded-xl"
          >
            <span className="text-sm font-bold text-primary px-3">{selectedFilenames.size} selected</span>
            <div className="h-6 w-px bg-primary/20" />
            <button
              onClick={() => handleBulkToggle(true)}
              className="p-2 text-primary hover:bg-primary/10 rounded-lg transition-colors"
              title="Enable Selected"
            >
              <Power size={18} />
            </button>
            <button
              onClick={() => handleBulkToggle(false)}
              className="p-2 text-primary hover:bg-primary/10 rounded-lg transition-colors"
              title="Disable Selected"
            >
              <Power size={18} className="rotate-180" />
            </button>
            <ConfirmDropdown
              onConfirm={() => handleBulkDelete(false)}
              title="Uninstall Selected?"
              message={`Are you sure you want to uninstall ${selectedFilenames.size} mods?`}
              confirmText="Uninstall"
              className="right-0"
            >
              <button className="p-2 text-red-400 hover:bg-red-400/10 rounded-lg transition-colors">
                <Trash2 size={18} />
              </button>
            </ConfirmDropdown>
          </motion.div>
        )}
      </div>

      {loading ? (
        <div className="flex flex-col items-center justify-center py-20 text-gray-500">
          <RefreshCw size={48} className="animate-spin mb-4 opacity-20" />
          <p className="font-medium animate-pulse">Loading mods...</p>
        </div>
      ) : filteredMods.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 bg-white/5 rounded-3xl border border-dashed border-white/10">
          <div className="w-20 h-20 bg-white/5 rounded-full flex items-center justify-center mb-4">
            <Layers size={40} className="text-gray-600" />
          </div>
          <h3 className="text-xl font-bold text-gray-400">No mods found</h3>
          <p className="text-gray-500 max-w-xs text-center mt-2">
            {searchQuery ? `No mods matching "${searchQuery}"` : "This instance doesn't have any mods installed yet."}
          </p>
        </div>
      ) : viewMode === 'grid' ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {filteredMods.map((mod) => (
            <ModGridCard
              key={mod.filename}
              mod={mod}
              isSelected={selectedFilenames.has(mod.filename)}
              onToggleSelect={() => toggleSelection(mod.filename)}
              onToggleEnabled={() => handleToggleMod(mod)}
              onDelete={(delConfig) => handleDeleteMod(mod, delConfig)}
              onConfigure={() => setConfiguringMod(mod)}
            />
          ))}
        </div>
      ) : (
        <div className="bg-white/5 rounded-3xl border border-white/5 overflow-hidden">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-white/5 border-b border-white/5">
                <th className="p-4 w-10">
                  <button
                    onClick={toggleAll}
                    className={`p-1 rounded-md transition-colors ${selectedFilenames.size === filteredMods.length ? 'text-primary' : 'text-gray-600 hover:text-gray-400'}`}
                  >
                    {selectedFilenames.size === filteredMods.length ? <CheckSquare size={18} /> : <Square size={18} />}
                  </button>
                </th>
                <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Mod</th>
                <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Version</th>
                <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Loader</th>
                <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Status</th>
                <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredMods.map((mod) => (
                <ModTableRow
                  key={mod.filename}
                  mod={mod}
                  isSelected={selectedFilenames.has(mod.filename)}
                  onToggleSelect={() => toggleSelection(mod.filename)}
                  onToggleEnabled={() => handleToggleMod(mod)}
                  onDelete={(delConfig) => handleDeleteMod(mod, delConfig)}
                  onConfigure={() => setConfiguringMod(mod)}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      <AnimatePresence>
        {configuringMod && (
          <ModConfigModal
            mod={configuringMod}
            instanceId={instanceId}
            onClose={() => setConfiguringMod(null)}
          />
        )}
      </AnimatePresence>
    </div>
  )
}

function ModGridCard({ mod, isSelected, onToggleSelect, onToggleEnabled, onDelete, onConfigure }: {
  mod: InstalledMod;
  isSelected: boolean;
  onToggleSelect: () => void;
  onToggleEnabled: () => void;
  onDelete: (delConfig: boolean) => void;
  onConfigure: () => void;
}) {
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
            <h4 className="font-bold truncate text-gray-200">{mod.name}</h4>
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

function ModTableRow({ mod, isSelected, onToggleSelect, onToggleEnabled, onDelete, onConfigure }: {
  mod: InstalledMod;
  isSelected: boolean;
  onToggleSelect: () => void;
  onToggleEnabled: () => void;
  onDelete: (delConfig: boolean) => void;
  onConfigure: () => void;
}) {
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
            <div className="font-bold text-gray-200">{mod.name}</div>
            <div className="text-xs text-gray-500 truncate max-w-[200px]">{mod.filename}</div>
          </div>
        </div>
      </td>
      <td className="p-4 text-sm text-gray-400">
        {mod.version || 'Unknown'}
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
