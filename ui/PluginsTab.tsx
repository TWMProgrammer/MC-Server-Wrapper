import { useState } from 'react'
import {
  Puzzle,
  Plus,
} from 'lucide-react'
import { InstalledPlugins } from './plugins/InstalledPlugins'
import { MarketplaceModal } from './plugins/MarketplaceModal'
import { DatabaseExplorerModal } from './database/DatabaseExplorerModal'
import { AnimatePresence } from 'framer-motion'

interface PluginsTabProps {
  instanceId: string;
}

export function PluginsTab({ instanceId }: PluginsTabProps) {
  const [isMarketplaceOpen, setIsMarketplaceOpen] = useState(false)
  const [isDbExplorerOpen, setIsDbExplorerOpen] = useState(false)
  const [refreshTrigger, setRefreshTrigger] = useState(0)

  const handleInstallSuccess = () => {
    setRefreshTrigger(prev => prev + 1)
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div>
          <h2 className="text-3xl font-black flex items-center gap-3 tracking-tight">
            <Puzzle className="text-primary" size={32} />
            Plugins
          </h2>
          <p className="text-gray-500 text-sm mt-1 font-medium">
            Manage your server's plugins and extensions.
          </p>
        </div>

        <button
          onClick={() => setIsMarketplaceOpen(true)}
          className="flex items-center gap-2 px-6 py-3 bg-primary text-white rounded-2xl font-bold shadow-xl shadow-primary/20 hover:bg-primary/90 transition-all hover:scale-[1.02] active:scale-[0.98]"
        >
          <Plus size={20} />
          Add Plugins
        </button>
      </div>

      <div className="min-h-[500px]">
        <InstalledPlugins 
          instanceId={instanceId} 
          refreshTrigger={refreshTrigger}
          onOpenDatabaseExplorer={() => setIsDbExplorerOpen(true)}
        />
      </div>

      <AnimatePresence>
        {isMarketplaceOpen && (
          <MarketplaceModal
            instanceId={instanceId}
            onClose={() => setIsMarketplaceOpen(false)}
            onInstallSuccess={handleInstallSuccess}
          />
        )}
        {isDbExplorerOpen && (
          <DatabaseExplorerModal
            instanceId={instanceId}
            onClose={() => setIsDbExplorerOpen(false)}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
