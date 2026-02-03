import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  History,
  Plus,
  Trash2,
  RefreshCw,
  Download,
  FileArchive,
  Search,
  Clock,
  ExternalLink,
  Loader2
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { BackupInfo } from './types'
import { useToast } from './hooks/useToast'
import { ConfirmDropdown } from './components/ConfirmDropdown'
import { formatSize } from './utils'

interface BackupsTabProps {
  instanceId: string;
}

interface BackupProgressPayload {
  instance_id: string;
  current: number;
  total: number;
  message: string;
}

export function BackupsTab({ instanceId }: BackupsTabProps) {
  const [backups, setBackups] = useState<BackupInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const { showToast } = useToast()

  useEffect(() => {
    loadBackups()
  }, [instanceId])

  useEffect(() => {
    const unlisten = listen<BackupProgressPayload>('backup-progress', (event) => {
      if (event.payload.instance_id === instanceId) {
        const progress = Math.round((event.payload.current / event.payload.total) * 100);
        setBackups(prev => prev.map(b =>
          b.status === 'creating' ? { ...b, progress } : b
        ));
      }
    });

    return () => {
      unlisten.then(u => u());
    };
  }, [instanceId]);

  const loadBackups = async () => {
    setLoading(true)
    try {
      const result = await invoke<BackupInfo[]>('list_backups', { instanceId })
      setBackups(result.map(b => ({ ...b, status: 'ready' as const })))
    } catch (err) {
      console.error('Failed to load backups:', err)
      showToast('Failed to load backups', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateBackup = async () => {
    const tempName = `Creating backup...`;
    const optimisticBackup: BackupInfo = {
      name: tempName,
      path: '',
      size: 0,
      created_at: new Date().toISOString(),
      status: 'creating',
      progress: 0
    };

    setBackups(prev => [optimisticBackup, ...prev]);
    setCreating(true)
    try {
      // Just send a simple prefix, the backend will add a clean timestamp
      await invoke('create_backup', { instanceId, name: 'Manual' })
      await loadBackups()
      showToast('Backup created successfully')
    } catch (err) {
      console.error('Failed to create backup:', err)
      showToast(`Error: ${err}`, 'error')
      setBackups(prev => prev.filter(b => b.name !== tempName));
    } finally {
      setCreating(false)
    }
  }

  const handleOpenBackup = async (backupName: string) => {
    try {
      await invoke('open_backup', { instanceId, backupName })
    } catch (err) {
      console.error('Failed to open backup:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleDeleteBackup = async (backupName: string) => {
    try {
      await invoke('delete_backup', { instanceId, backupName })
      await loadBackups()
      showToast('Backup deleted successfully')
    } catch (err) {
      console.error('Failed to delete backup:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const handleRestoreBackup = async (backupName: string) => {
    setLoading(true)
    try {
      await invoke('restore_backup', { instanceId, backupName })
      showToast('Backup restored successfully')
    } catch (err) {
      console.error('Failed to restore backup:', err)
      showToast(`Error: ${err}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  const filteredBackups = backups.filter(b =>
    b.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-2">
            <History className="text-primary" />
            Backups
          </h2>
          <p className="text-gray-500 text-sm mt-1">
            Manage your server snapshots and restore points.
          </p>
        </div>

        <button
          onClick={handleCreateBackup}
          disabled={creating || loading}
          className="flex items-center justify-center gap-2 px-6 py-2.5 bg-primary hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl transition-all font-medium shadow-lg shadow-primary/20"
        >
          {creating ? (
            <RefreshCw size={18} className="animate-spin" />
          ) : (
            <Plus size={18} />
          )}
          Create Backup
        </button>
      </div>

      <div className="bg-surface border border-white/5 rounded-2xl overflow-hidden">
        <div className="p-4 border-b border-white/5 flex items-center gap-4 bg-white/5">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
            <input
              type="text"
              placeholder="Search backups..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-black/20 border border-white/5 rounded-lg focus:outline-none focus:border-primary/50 transition-colors"
            />
          </div>
          <button
            onClick={loadBackups}
            className="p-2 hover:bg-white/5 rounded-lg transition-colors text-gray-400"
            title="Refresh list"
          >
            <RefreshCw size={20} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left">
            <thead>
              <tr className="text-gray-500 text-sm uppercase tracking-wider">
                <th className="px-6 py-4 font-semibold">Name</th>
                <th className="px-6 py-4 font-semibold">Date</th>
                <th className="px-6 py-4 font-semibold">Size</th>
                <th className="px-6 py-4 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {filteredBackups.length > 0 ? (
                filteredBackups.map((backup) => (
                  <tr key={backup.name} className="hover:bg-white/5 transition-colors group">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <div className="p-2 bg-primary/10 rounded-lg text-primary">
                          {backup.status === 'creating' ? (
                            <Loader2 size={20} className="animate-spin" />
                          ) : (
                            <FileArchive size={20} />
                          )}
                        </div>
                        <div className="flex flex-col">
                          <span className="font-medium">{backup.name}</span>
                          {backup.status === 'creating' && (
                            <div className="mt-2 w-48">
                              <div className="flex justify-between text-[10px] mb-1">
                                <span className="text-primary font-bold">CREATING...</span>
                                <span className="text-gray-400">{backup.progress || 0}%</span>
                              </div>
                              <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                                <motion.div
                                  initial={{ width: 0 }}
                                  animate={{ width: `${backup.progress || 0}%` }}
                                  className="h-full bg-primary"
                                />
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-gray-400 text-sm">
                      <div className="flex items-center gap-2">
                        <Clock size={14} />
                        {new Date(backup.created_at).toLocaleString()}
                      </div>
                    </td>
                    <td className="px-6 py-4 text-gray-400 text-sm">
                      {backup.status === 'creating' ? 'Calculating...' : formatSize(backup.size)}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2">
                        {backup.status !== 'creating' && (
                          <>
                            <button
                              onClick={() => handleOpenBackup(backup.name)}
                              className="p-2 hover:bg-blue-500/20 text-blue-500 rounded-lg transition-all hover:scale-110 active:scale-95"
                              title="Open backup file"
                            >
                              <ExternalLink size={18} />
                            </button>
                            <ConfirmDropdown
                              title="Restore Backup"
                              message={`Are you sure you want to restore "${backup.name}"? This will overwrite all current server files.`}
                              onConfirm={() => handleRestoreBackup(backup.name)}
                              confirmText="Restore"
                              variant="warning"
                            >
                              <button
                                className="p-2 hover:bg-green-500/20 text-green-500 rounded-lg transition-all hover:scale-110 active:scale-95"
                                title="Restore this backup"
                              >
                                <Download size={18} />
                              </button>
                            </ConfirmDropdown>
                            <ConfirmDropdown
                              title="Delete Backup"
                              message={`Are you sure you want to delete "${backup.name}"? This action cannot be undone.`}
                              onConfirm={() => handleDeleteBackup(backup.name)}
                              confirmText="Delete"
                              variant="danger"
                            >
                              <button
                                className="p-2 hover:bg-red-500/20 text-red-500 rounded-lg transition-all hover:scale-110 active:scale-95"
                                title="Delete backup"
                              >
                                <Trash2 size={18} />
                              </button>
                            </ConfirmDropdown>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={4} className="px-6 py-12 text-center text-gray-500">
                    {loading ? (
                      <div className="flex flex-col items-center gap-3">
                        <RefreshCw className="animate-spin text-primary" size={32} />
                        <p>Loading backups...</p>
                      </div>
                    ) : (
                      <div className="flex flex-col items-center gap-3">
                        <History size={48} className="opacity-20" />
                        <p>{searchQuery ? 'No backups matching your search' : 'No backups found for this instance'}</p>
                      </div>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
