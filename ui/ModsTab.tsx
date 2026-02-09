import { useState } from 'react'
import {
  Layers,
  Plus
} from 'lucide-react'
import { InstalledMods } from './mods/InstalledMods'
import { AnimatePresence } from 'framer-motion'
import { ModMarketplaceModal } from './mods/ModMarketplaceModal'

interface ModsTabProps {
  instanceId: string;
}

export function ModsTab({ instanceId }: ModsTabProps) {
  const [isMarketplaceOpen, setIsMarketplaceOpen] = useState(false)
  const [refreshTrigger, setRefreshTrigger] = useState(0)

  const handleInstallSuccess = () => {
    setRefreshTrigger(prev => prev + 1)
    setIsMarketplaceOpen(false)
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div>
          <h2 className="text-3xl font-black flex items-center gap-3 tracking-tight">
            <Layers className="text-primary" size={32} />
            Mods
          </h2>
          <p className="text-gray-500 text-sm mt-1 font-medium">
            Manage your server's mods and configurations.
          </p>
        </div>

        <button
          onClick={() => setIsMarketplaceOpen(true)}
          className="flex items-center gap-2 px-6 py-3 bg-primary text-white rounded-2xl font-bold shadow-xl shadow-primary/20 hover:bg-primary/90 transition-all hover:scale-[1.02] active:scale-[0.98]"
        >
          <Plus size={20} />
          Add Mods
        </button>
      </div>

      <div className="min-h-[500px]">
        <InstalledMods 
          instanceId={instanceId} 
          refreshTrigger={refreshTrigger} 
        />
      </div>

      <AnimatePresence>
        {isMarketplaceOpen && (
          <ModMarketplaceModal 
            instanceId={instanceId} 
            onClose={() => setIsMarketplaceOpen(false)}
            onInstallSuccess={handleInstallSuccess}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
