import { Database, Trash2, RefreshCw, HardDrive, FileText, Clock } from 'lucide-react'
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Section } from './SettingsShared'
import { formatSize } from '../../utils'
import { motion, AnimatePresence } from 'framer-motion'
import { useToast } from '../../hooks/useToast'

interface AssetCacheStats {
  count: number;
  total_size: number;
}

export function CacheSettings() {
  const [stats, setStats] = useState<AssetCacheStats | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isCleaning, setIsCleaning] = useState(false)
  const { showToast } = useToast()

  const fetchStats = async () => {
    try {
      const result = await invoke<AssetCacheStats>('get_asset_cache_stats')
      setStats(result)
    } catch (err) {
      console.error('Failed to fetch cache stats:', err)
      showToast('Failed to fetch cache statistics', 'error')
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    fetchStats()
  }, [])

  const handleCleanup = async (days: number | null) => {
    setIsCleaning(true)
    try {
      const count = await invoke<number>('cleanup_assets', { maxAgeDays: days })
      showToast(`Cleaned up ${count} cached assets`, 'success')
      await fetchStats()
    } catch (err) {
      console.error('Failed to cleanup assets:', err)
      showToast('Failed to cleanup assets', 'error')
    } finally {
      setIsCleaning(false)
    }
  }

  return (
    <div className="space-y-8">
      <Section title="Asset Cache" icon={Database}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-6 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5 flex items-center gap-4">
            <div className="p-3 bg-primary/10 rounded-xl text-primary">
              <HardDrive size={24} />
            </div>
            <div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Total Size</div>
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {isLoading ? (
                  <div className="h-8 w-24 bg-black/10 dark:bg-white/10 animate-pulse rounded" />
                ) : (
                  formatSize(stats?.total_size || 0)
                )}
              </div>
            </div>
          </div>

          <div className="p-6 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5 flex items-center gap-4">
            <div className="p-3 bg-blue-500/10 rounded-xl text-blue-500">
              <FileText size={24} />
            </div>
            <div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Cached Files</div>
              <div className="text-2xl font-bold text-gray-900 dark:text-white">
                {isLoading ? (
                  <div className="h-8 w-16 bg-black/10 dark:bg-white/10 animate-pulse rounded" />
                ) : (
                  stats?.count || 0
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="p-6 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5 space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-bold text-gray-900 dark:text-white">Maintenance</div>
              <div className="text-xs text-gray-500 mt-1">Free up space by removing old or all cached assets</div>
            </div>
            <button
              onClick={fetchStats}
              disabled={isLoading || isCleaning}
              className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg transition-colors text-gray-500 hover:text-primary"
              title="Refresh statistics"
            >
              <RefreshCw size={18} className={isLoading ? 'animate-spin' : ''} />
            </button>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <button
              onClick={() => handleCleanup(7)}
              disabled={isCleaning}
              className="flex items-center gap-3 p-4 rounded-xl border border-black/5 dark:border-white/5 hover:border-primary/30 bg-black/5 dark:bg-white/5 transition-all group"
            >
              <div className="p-2 bg-black/5 dark:bg-white/5 rounded-lg text-gray-500 group-hover:text-primary">
                <Clock size={18} />
              </div>
              <div className="text-left">
                <div className="text-sm font-bold text-gray-700 dark:text-gray-200">Clean Old Assets</div>
                <div className="text-[10px] text-gray-500 mt-0.5">Remove files older than 7 days</div>
              </div>
            </button>

            <button
              onClick={() => handleCleanup(0)}
              disabled={isCleaning}
              className="flex items-center gap-3 p-4 rounded-xl border border-black/5 dark:border-white/5 hover:border-red-500/30 bg-black/5 dark:bg-white/5 transition-all group"
            >
              <div className="p-2 bg-red-500/10 rounded-lg text-red-500">
                <Trash2 size={18} />
              </div>
              <div className="text-left">
                <div className="text-sm font-bold text-gray-700 dark:text-gray-200">Clear All Cache</div>
                <div className="text-[10px] text-gray-500 mt-0.5">Completely empty the asset cache</div>
              </div>
            </button>
          </div>
        </div>
      </Section>

      <Section title="Information" icon={Clock}>
        <div className="p-4 bg-primary/5 rounded-xl border border-primary/10">
          <p className="text-xs text-primary leading-relaxed">
            The asset cache stores marketplace icons, screenshots, and player heads locally to improve performance and reduce network usage.
            Files are automatically cleaned up if they haven't been accessed for more than 7 days.
          </p>
        </div>
      </Section>
    </div>
  )
}
