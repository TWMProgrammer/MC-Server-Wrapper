import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { 
  History, 
  Plus, 
  Trash2, 
  RefreshCw, 
  Download, 
  FileArchive,
  Search,
  AlertTriangle,
  CheckCircle2,
  Clock
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { BackupInfo } from './types'

interface BackupsTabProps {
  instanceId: string;
}

export function BackupsTab({ instanceId }: BackupsTabProps) {
  const [backups, setBackups] = useState<BackupInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  useEffect(() => {
    loadBackups()
  }, [instanceId])

  const loadBackups = async () => {
    setLoading(true)
    try {
      const result = await invoke<BackupInfo[]>('list_backups', { instanceId })
      setBackups(result)
      setError(null)
    } catch (err) {
      console.error('Failed to load backups:', err)
      setError('Failed to load backups')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateBackup = async () => {
    setCreating(true)
    setError(null)
    try {
      const name = `backup_${new Date().toISOString().replace(/[:.]/g, '-')}`
      await invoke('create_backup', { instanceId, name })
      await loadBackups()
      setSuccess('Backup created successfully')
      setTimeout(() => setSuccess(null), 3000)
    } catch (err) {
      console.error('Failed to create backup:', err)
      setError(err as string)
    } finally {
      setCreating(false)
    }
  }

  const handleDeleteBackup = async (backupName: string) => {
    if (!confirm(`Are you sure you want to delete backup "${backupName}"?`)) return
    
    try {
      await invoke('delete_backup', { instanceId, backupName })
      await loadBackups()
      setSuccess('Backup deleted successfully')
      setTimeout(() => setSuccess(null), 3000)
    } catch (err) {
      console.error('Failed to delete backup:', err)
      setError(err as string)
    }
  }

  const handleRestoreBackup = async (backupName: string) => {
    if (!confirm(`WARNING: Restoring backup "${backupName}" will overwrite all current server files. Are you sure?`)) return
    
    setLoading(true)
    try {
      await invoke('restore_backup', { instanceId, backupName })
      setSuccess('Backup restored successfully')
      setTimeout(() => setSuccess(null), 3000)
    } catch (err) {
      console.error('Failed to restore backup:', err)
      setError(err as string)
    } finally {
      setLoading(false)
    }
  }

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
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

      {/* Status Messages */}
      <AnimatePresence>
        {error && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="bg-red-500/10 border border-red-500/20 text-red-500 px-4 py-3 rounded-xl flex items-center gap-3"
          >
            <AlertTriangle size={18} />
            <span className="text-sm font-medium">{error}</span>
          </motion.div>
        )}
        {success && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="bg-green-500/10 border border-green-500/20 text-green-500 px-4 py-3 rounded-xl flex items-center gap-3"
          >
            <CheckCircle2 size={18} />
            <span className="text-sm font-medium">{success}</span>
          </motion.div>
        )}
      </AnimatePresence>

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
                          <FileArchive size={20} />
                        </div>
                        <span className="font-medium">{backup.name}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-gray-400 text-sm">
                      <div className="flex items-center gap-2">
                        <Clock size={14} />
                        {new Date(backup.created_at).toLocaleString()}
                      </div>
                    </td>
                    <td className="px-6 py-4 text-gray-400 text-sm">
                      {formatSize(backup.size)}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button
                          onClick={() => handleRestoreBackup(backup.name)}
                          className="p-2 hover:bg-green-500/20 text-green-500 rounded-lg transition-colors"
                          title="Restore this backup"
                        >
                          <Download size={18} />
                        </button>
                        <button
                          onClick={() => handleDeleteBackup(backup.name)}
                          className="p-2 hover:bg-red-500/20 text-red-500 rounded-lg transition-colors"
                          title="Delete backup"
                        >
                          <Trash2 size={18} />
                        </button>
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
