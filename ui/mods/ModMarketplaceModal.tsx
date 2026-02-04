import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { X, Sparkles } from 'lucide-react'
import { ModMarketplace } from './ModMarketplace'
import { useAppSettings } from '../hooks/useAppSettings'

interface ModMarketplaceModalProps {
  instanceId: string;
  onClose: () => void;
  onInstallSuccess?: () => void;
}

export function ModMarketplaceModal({ instanceId, onClose, onInstallSuccess }: ModMarketplaceModalProps) {
  const { settings } = useAppSettings()

  return createPortal(
    <div
      className="fixed inset-0 z-[100] overflow-hidden"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
      <div className="w-full h-full flex items-center justify-center p-4 md:p-8">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/80 backdrop-blur-md"
        />

        <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 10 }}
          className="relative w-[80%] h-[90%] bg-surface border border-white/10 rounded-[2.5rem] shadow-2xl overflow-hidden flex flex-col"
        >
          {/* Header */}
          <div className="p-8 border-b border-white/5 flex items-center justify-between bg-white/5">
            <div>
              <h2 className="text-3xl font-black flex items-center gap-3 tracking-tight">
                <Sparkles className="text-primary" size={32} />
                Mod Marketplace
              </h2>
              <p className="text-gray-500 text-sm mt-1 font-medium">
                Discover and install new mods from Modrinth and CurseForge.
              </p>
            </div>

            <button
              onClick={onClose}
              className="p-3 hover:bg-white/10 text-gray-500 hover:text-white rounded-2xl transition-all"
            >
              <X size={28} />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-hidden p-8 min-h-0 flex flex-col">
            <ModMarketplace instanceId={instanceId} onInstallSuccess={onInstallSuccess} />
          </div>
        </motion.div>
      </div>
    </div>,
    document.body
  )
}
