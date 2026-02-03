import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Puzzle,
  RefreshCw,
  CheckSquare,
  Square
} from 'lucide-react'
import { AnimatePresence } from 'framer-motion'
import { InstalledPlugin, PluginUpdate } from '../types'
import { useToast } from '../hooks/useToast'
import { PluginConfigModal } from './PluginConfigModal'
import { PluginCard } from './PluginCard'
import { PluginTableRow } from './PluginTableRow'
import { PluginFilters } from './PluginFilters'
import { BulkActions } from './BulkActions'

interface InstalledPluginsProps {
  instanceId: string;
  refreshTrigger?: number;
}

type ViewMode = 'table' | 'grid'

export function InstalledPlugins({ instanceId, refreshTrigger }: InstalledPluginsProps) {
  const [plugins, setPlugins] = useState<InstalledPlugin[]>([])
  const [updates, setUpdates] = useState<PluginUpdate[]>([])
  const [loading, setLoading] = useState(true)
  const [checkingUpdates, setCheckingUpdates] = useState(false)
  const [updatingPlugins, setUpdatingPlugins] = useState<Set<string>>(new Set())
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<ViewMode>('table')
  const [selectedPlugin, setSelectedPlugin] = useState<InstalledPlugin | null>(null)
  const [isConfigOpen, setIsConfigOpen] = useState(false)
  const [selectedFilenames, setSelectedFilenames] = useState<Set<string>>(new Set())
  const { showToast } = useToast()

  useEffect(() => {
    loadPlugins()
  }, [instanceId, refreshTrigger])

  const loadPlugins = async () => {
    setLoading(true)
    try {
      const result = await invoke<InstalledPlugin[]>('list_installed_plugins', { instanceId })
      setPlugins(result)
      // Reset selection when list changes
      setSelectedFilenames(new Set())
    } catch (err) {
      console.error('Failed to load plugins:', err)
      showToast('Failed to load plugins', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true)
    try {
      const result = await invoke<PluginUpdate[]>('check_for_plugin_updates', { instanceId })
      setUpdates(result)
      if (result.length > 0) {
        showToast(`Found ${result.length} updates!`, 'info')
      } else {
        showToast('All plugins are up to date', 'info')
      }
    } catch (err) {
      console.error('Failed to check for updates:', err)
      showToast('Failed to check for updates', 'error')
    } finally {
      setCheckingUpdates(false)
    }
  }

  const handleUpdatePlugin = async (update: PluginUpdate) => {
    setUpdatingPlugins(prev => new Set(prev).add(update.filename))
    try {
      await invoke('update_plugin', {
        instanceId,
        filename: update.filename,
        projectId: update.project_id,
        provider: update.provider,
        latestVersionId: update.latest_version_id
      })
      showToast(`Updated ${update.filename} to ${update.latest_version}`)
      setUpdates(prev => prev.filter(u => u.filename !== update.filename))
      await loadPlugins()
    } catch (err) {
      console.error('Failed to update plugin:', err)
      showToast(`Failed to update ${update.filename}: ${err}`, 'error')
    } finally {
      setUpdatingPlugins(prev => {
        const next = new Set(prev)
        next.delete(update.filename)
        return next
      })
    }
  }

  const handleOpenConfig = (plugin: InstalledPlugin) => {
    setSelectedPlugin(plugin)
    setIsConfigOpen(true)
  }

  const handleTogglePlugin = async (plugin: InstalledPlugin) => {
    try {
      await invoke('toggle_plugin', {
        instanceId,
        filename: plugin.filename,
        enable: !plugin.enabled
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

  const handleBulkToggle = async (enable: boolean) => {
    try {
      await invoke('bulk_toggle_plugins', {
        instanceId,
        filenames: Array.from(selectedFilenames),
        enable
      })
      showToast(`Bulk ${enable ? 'enable' : 'disable'} successful`)
      await loadPlugins()
    } catch (err) {
      showToast(`Bulk toggle failed: ${err}`, 'error')
    }
  }

  const handleBulkDelete = async (deleteConfig: boolean) => {
    try {
      await invoke('bulk_uninstall_plugins', {
        instanceId,
        filenames: Array.from(selectedFilenames),
        deleteConfig
      })
      showToast(`Bulk uninstall successful`)
      await loadPlugins()
    } catch (err) {
      showToast(`Bulk uninstall failed: ${err}`, 'error')
    }
  }

  const handleBulkUpdate = async () => {
    const updatesToRun = updates.filter(u => selectedFilenames.has(u.filename))
    if (updatesToRun.length === 0) return

    showToast(`Updating ${updatesToRun.length} plugins...`, 'info')

    for (const update of updatesToRun) {
      await handleUpdatePlugin(update)
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
    if (selectedFilenames.size === filteredPlugins.length) {
      setSelectedFilenames(new Set())
    } else {
      setSelectedFilenames(new Set(filteredPlugins.map(p => p.filename)))
    }
  }

  const filteredPlugins = plugins.filter(p =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    p.author?.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <PluginFilters
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        viewMode={viewMode}
        setViewMode={setViewMode}
        onCheckUpdates={handleCheckUpdates}
        onRefresh={loadPlugins}
        loading={loading}
        checkingUpdates={checkingUpdates}
      />

      <AnimatePresence>
        {selectedFilenames.size > 0 && (
          <BulkActions
            selectedCount={selectedFilenames.size}
            hasUpdates={updates.some(u => selectedFilenames.has(u.filename))}
            onBulkToggle={handleBulkToggle}
            onBulkUpdate={handleBulkUpdate}
            onBulkDelete={handleBulkDelete}
            onDeselect={() => setSelectedFilenames(new Set())}
          />
        )}
      </AnimatePresence>

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
                  <th className="px-6 py-4 font-semibold w-10">
                    <button
                      onClick={toggleAll}
                      className={`p-1 rounded transition-colors ${selectedFilenames.size === filteredPlugins.length && filteredPlugins.length > 0
                        ? 'text-primary'
                        : 'text-gray-600 hover:text-gray-400'
                        }`}
                    >
                      {selectedFilenames.size === filteredPlugins.length && filteredPlugins.length > 0
                        ? <CheckSquare size={18} />
                        : <Square size={18} />
                      }
                    </button>
                  </th>
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
                    <PluginTableRow
                      key={plugin.filename}
                      plugin={plugin}
                      isSelected={selectedFilenames.has(plugin.filename)}
                      update={updates.find(u => u.filename === plugin.filename)}
                      isUpdating={updatingPlugins.has(plugin.filename)}
                      onToggleSelection={toggleSelection}
                      onToggle={handleTogglePlugin}
                      onUpdate={handleUpdatePlugin}
                      onOpenConfig={handleOpenConfig}
                      onDelete={handleDeletePlugin}
                    />
                  ))}
                </AnimatePresence>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="p-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <AnimatePresence mode="popLayout">
              {filteredPlugins.map((plugin) => (
                <PluginCard
                  key={plugin.filename}
                  plugin={plugin}
                  isSelected={selectedFilenames.has(plugin.filename)}
                  update={updates.find(u => u.filename === plugin.filename)}
                  isUpdating={updatingPlugins.has(plugin.filename)}
                  onToggleSelection={toggleSelection}
                  onToggle={handleTogglePlugin}
                  onUpdate={handleUpdatePlugin}
                  onOpenConfig={handleOpenConfig}
                  onDelete={handleDeletePlugin}
                />
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
