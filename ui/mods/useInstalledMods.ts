import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { InstalledMod, ModUpdate } from '../types'
import { useToast } from '../hooks/useToast'

export function useInstalledMods(instanceId: string, refreshTrigger?: number) {
  const [mods, setMods] = useState<InstalledMod[]>([])
  const [updates, setUpdates] = useState<ModUpdate[]>([])
  const [loading, setLoading] = useState(true)
  const [checkingUpdates, setCheckingUpdates] = useState(false)
  const [updatingMods, setUpdatingMods] = useState<Set<string>>(new Set())
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<'table' | 'grid'>('grid')
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

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true)
    try {
      const result = await invoke<ModUpdate[]>('check_for_mod_updates', { instanceId })
      setUpdates(result)
      if (result.length > 0) {
        showToast(`Found ${result.length} updates!`, 'info')
      } else {
        showToast('All mods are up to date', 'info')
      }
    } catch (err) {
      console.error('Failed to check for updates:', err)
      showToast('Failed to check for updates', 'error')
    } finally {
      setCheckingUpdates(false)
    }
  }

  const handleUpdateMod = async (update: ModUpdate) => {
    setUpdatingMods(prev => new Set(prev).add(update.filename))
    try {
      await invoke('update_mod', {
        instanceId,
        filename: update.filename,
        projectId: update.project_id,
        provider: update.provider,
        latestVersionId: update.latest_version_id
      })
      showToast(`Updated ${update.filename} to ${update.latest_version}`)
      setUpdates(prev => prev.filter(u => u.filename !== update.filename))
      await loadMods()
    } catch (err) {
      console.error('Failed to update mod:', err)
      showToast(`Failed to update ${update.filename}: ${err}`, 'error')
    } finally {
      setUpdatingMods(prev => {
        const next = new Set(prev)
        next.delete(update.filename)
        return next
      })
    }
  }

  const handleBulkUpdate = async () => {
    const updatesToRun = updates.filter(u => selectedFilenames.has(u.filename))
    if (updatesToRun.length === 0) return

    showToast(`Updating ${updatesToRun.length} mods...`, 'info')

    for (const update of updatesToRun) {
      await handleUpdateMod(update)
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

  const filteredMods = useMemo(() => {
    return mods.filter(m =>
      m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
      m.author?.toLowerCase().includes(searchQuery.toLowerCase())
    )
  }, [mods, searchQuery])

  const toggleAll = () => {
    if (selectedFilenames.size === filteredMods.length) {
      setSelectedFilenames(new Set())
    } else {
      setSelectedFilenames(new Set(filteredMods.map(p => p.filename)))
    }
  }

  return {
    mods,
    updates,
    loading,
    checkingUpdates,
    updatingMods,
    searchQuery,
    setSearchQuery,
    viewMode,
    setViewMode,
    selectedFilenames,
    setSelectedFilenames,
    configuringMod,
    setConfiguringMod,
    filteredMods,
    loadMods,
    handleCheckUpdates,
    handleUpdateMod,
    handleBulkUpdate,
    handleToggleMod,
    handleDeleteMod,
    handleBulkToggle,
    handleBulkDelete,
    toggleSelection,
    toggleAll
  }
}
