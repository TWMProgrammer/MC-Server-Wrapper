import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Puzzle,
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
  Sliders
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { InstalledPlugin } from '../types'
import { useToast } from '../hooks/useToast'
import { ConfirmDropdown } from '../components/ConfirmDropdown'
import { PluginConfigModal } from './PluginConfigModal'

interface InstalledPluginsProps {
  instanceId: string;
  refreshTrigger?: number;
}

type ViewMode = 'table' | 'grid'

export function InstalledPlugins({ instanceId, refreshTrigger }: InstalledPluginsProps) {
  const [plugins, setPlugins] = useState<InstalledPlugin[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<ViewMode>('table')
  const [selectedPlugin, setSelectedPlugin] = useState<InstalledPlugin | null>(null)
  const [isConfigOpen, setIsConfigOpen] = useState(false)
  const { showToast } = useToast()

  useEffect(() => {
    loadPlugins()
  }, [instanceId, refreshTrigger])

  const loadPlugins = async () => {
    setLoading(true)
    try {
      const result = await invoke<InstalledPlugin[]>('list_installed_plugins', { instanceId })
      setPlugins(result)
    } catch (err) {
      console.error('Failed to load plugins:', err)
      showToast('Failed to load plugins', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleTogglePlugin = async (plugin: InstalledPlugin) => {
    try {
      await invoke('toggle_plugin', {
        instanceId,
        filename: plugin.filename,
        enabled: !plugin.enabled
      })
      showToast(`Plugin ${!plugin.enabled ? 'enabled' : 'disabled'} successfully`)
      await loadPlugins()
    } catch (err) {
      console.error('Failed to toggle plugin:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleDeletePlugin = async (plugin: InstalledPlugin, deleteConfig: boolean) => {
    try {
      await invoke('uninstall_plugin', {
        instanceId,
        filename: plugin.filename,
        deleteConfig
      })
      showToast('Plugin uninstalled successfully')
      await loadPlugins()
    } catch (err) {
      console.error('Failed to uninstall plugin:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleOpenConfig = (plugin: InstalledPlugin) => {
    setSelectedPlugin(plugin)
    setIsConfigOpen(true)
  }

  const filteredPlugins = plugins.filter(p =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.author?.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
            <input
              type="text"
              placeholder="Search installed plugins..."
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
            onClick={loadPlugins}
            disabled={loading}
            className="p-2.5 bg-white/5 hover:bg-white/10 text-gray-400 rounded-xl transition-all border border-white/5"
            title="Refresh list"
          >
            <RefreshCw size={20} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
      </div>

      <div className="bg-surface border border-white/5 rounded-2xl overflow-hidden">
        {loading ? (
          <div className="py-20 flex flex-col items-center justify-center">
            <RefreshCw size={40} className="animate-spin text-primary opacity-50 mb-4" />
            <p className="text-gray-500">Loading plugins...</p>
          </div>
        ) : filteredPlugins.length === 0 ? (
          <div className="py-20 flex flex-col items-center justify-center text-center px-4">
            <div className="p-6 bg-white/5 rounded-3xl mb-4">
              <Puzzle size={48} className="text-gray-600" />
            </div>
            <h3 className="text-xl font-bold text-white mb-2">
              {searchQuery ? "No plugins found" : "No plugins installed"}
            </h3>
            <p className="text-gray-500 max-w-xs">
              {searchQuery ? "Try a different search term." : "Your server doesn't have any plugins yet."}
            </p>
          </div>
        ) : viewMode === 'table' ? (
          <div className="overflow-x-auto">
            <table className="w-full text-left">
              <thead>
                <tr className="text-gray-500 text-sm uppercase tracking-wider">
                  <th className="px-6 py-4 font-semibold">Plugin</th>
                  <th className="px-6 py-4 font-semibold">Status</th>
                  <th className="px-6 py-4 font-semibold">Author</th>
                  <th className="px-6 py-4 font-semibold">Version</th>
                  <th className="px-6 py-4 font-semibold text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                <AnimatePresence mode="popLayout">
                  {filteredPlugins.map((plugin) => (
                    <motion.tr
                      key={plugin.filename}
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      className={`hover:bg-white/5 transition-colors ${!plugin.enabled ? 'opacity-60 grayscale-[0.5]' : ''}`}
                    >
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-3">
                          <div className={`p-2 rounded-lg ${plugin.enabled ? 'bg-primary/10 text-primary' : 'bg-gray-500/10 text-gray-500'}`}>
                            <Package size={20} />
                          </div>
                          <div>
                            <div className="font-medium flex items-center gap-2">
                              {plugin.name}
                              {!plugin.enabled && (
                                <span className="text-[10px] bg-gray-500/20 text-gray-400 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                                  Disabled
                                </span>
                              )}
                            </div>
                            <div className="text-xs text-gray-500 line-clamp-1 max-w-[300px]" title={plugin.description || plugin.filename}>
                              {plugin.description || plugin.filename}
                            </div>
                          </div>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <div className={`inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium ${plugin.enabled
                            ? 'bg-green-500/10 text-green-500'
                            : 'bg-red-500/10 text-red-500'
                          }`}>
                          <Power size={14} />
                          {plugin.enabled ? 'Enabled' : 'Disabled'}
                        </div>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-400">
                        {plugin.author ? (
                          <div className="flex items-center gap-1.5">
                            <User size={14} className="opacity-50" />
                            {plugin.author}
                          </div>
                        ) : '—'}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-400">
                        {plugin.version || 'Unknown'}
                      </td>
                      <td className="px-6 py-4 text-right">
                        <div className="flex items-center justify-end gap-2">
                          <button
                            onClick={() => handleTogglePlugin(plugin)}
                            className={`p-2 rounded-lg transition-colors ${plugin.enabled
                                ? 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20'
                                : 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
                              }`}
                            title={plugin.enabled ? 'Disable' : 'Enable'}
                          >
                            <Power size={16} />
                          </button>
                          <button
                            onClick={() => handleOpenConfig(plugin)}
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
                            onConfirm={() => handleDeletePlugin(plugin, false)}
                          >
                            <button className="p-2 bg-red-500/10 text-red-500 hover:bg-red-500/20 rounded-lg transition-colors">
                              <Trash2 size={16} />
                            </button>
                          </ConfirmDropdown>
                        </div>
                      </td>
                    </motion.tr>
                  ))}
                </AnimatePresence>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="p-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <AnimatePresence mode="popLayout">
              {filteredPlugins.map((plugin) => (
                <motion.div
                  key={plugin.filename}
                  layout
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.9 }}
                  className={`bg-white/5 border border-white/5 rounded-2xl p-4 hover:border-primary/30 transition-all group flex flex-col h-full ${!plugin.enabled ? 'opacity-60 grayscale-[0.5]' : ''}`}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className={`p-3 rounded-xl ${plugin.enabled ? 'bg-primary/10 text-primary' : 'bg-gray-500/10 text-gray-500'}`}>
                      <Package size={24} />
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleTogglePlugin(plugin)}
                        className={`p-2 rounded-lg transition-colors ${plugin.enabled
                            ? 'bg-orange-500/10 text-orange-500 hover:bg-orange-500/20'
                            : 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
                          }`}
                        title={plugin.enabled ? 'Disable' : 'Enable'}
                      >
                        <Power size={16} />
                      </button>
                      <button
                        onClick={() => handleOpenConfig(plugin)}
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
                        onConfirm={() => handleDeletePlugin(plugin, false)}
                      >
                        <button className="p-2 bg-red-500/10 text-red-500 hover:bg-red-500/20 rounded-lg transition-colors">
                          <Trash2 size={16} />
                        </button>
                      </ConfirmDropdown>
                    </div>
                  </div>

                  <div className="flex-1">
                    <div className="font-bold text-lg flex items-center gap-2 mb-1">
                      {plugin.name}
                      {!plugin.enabled && (
                        <span className="text-[10px] bg-gray-500/20 text-gray-400 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 line-clamp-2 mb-4 h-10">
                      {plugin.description || "No description available."}
                    </p>
                  </div>

                  <div className="flex items-center justify-between mt-4 pt-4 border-t border-white/5 text-xs text-gray-400">
                    <div className="flex items-center gap-3">
                      <span className="flex items-center gap-1">
                        <Info size={12} className="opacity-50" />
                        v{plugin.version || '?.?.?'}
                      </span>
                      {plugin.author && (
                        <span className="flex items-center gap-1 truncate max-w-[100px]">
                          <User size={12} className="opacity-50" />
                          {plugin.author}
                        </span>
                      )}
                    </div>
                    <div className="font-mono opacity-30 text-[10px] truncate max-w-[100px]">
                      {plugin.filename}
                    </div>
                  </div>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      <AnimatePresence>
        {isConfigOpen && selectedPlugin && (
          <PluginConfigModal
            plugin={selectedPlugin}
            instanceId={instanceId}
            onClose={() => setIsConfigOpen(false)}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
