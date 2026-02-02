import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { X, Check, Download, RefreshCw, Package, AlertCircle } from 'lucide-react'
import { Project } from '../types'
import { useState, useEffect } from 'react'
import { useAppSettings } from '../hooks/useAppSettings'

interface ModReviewModalProps {
  selectedMods: Project[];
  preFetchedDependencies?: Project[];
  instanceId: string;
  onClose: () => void;
  onConfirm: (mods: Project[]) => Promise<void>;
  isInstalling: boolean;
}

export function ModReviewModal({
  selectedMods,
  preFetchedDependencies = [],
  instanceId,
  onClose,
  onConfirm,
  isInstalling
}: ModReviewModalProps) {
  const [dependencies, setDependencies] = useState<Project[]>(preFetchedDependencies)
  const [selectedForInstall, setSelectedForInstall] = useState<Set<string>>(new Set())
  const { settings } = useAppSettings()

  useEffect(() => {
    // Initialize with all selected mods and pre-fetched dependencies immediately
    const allIds = new Set([
      ...selectedMods.map(m => m.id),
      ...preFetchedDependencies.map(m => m.id)
    ])
    setSelectedForInstall(allIds)
    setDependencies(preFetchedDependencies)
  }, [selectedMods, preFetchedDependencies])

  const toggleSelection = (id: string) => {
    const newSelection = new Set(selectedForInstall)
    if (newSelection.has(id)) {
      newSelection.delete(id)
    } else {
      newSelection.add(id)
    }
    setSelectedForInstall(newSelection)
  }

  const handleConfirm = () => {
    const modsToInstall = [
      ...selectedMods.filter(m => selectedForInstall.has(m.id)),
      ...dependencies.filter(m => selectedForInstall.has(m.id))
    ]
    onConfirm(modsToInstall)
  }

  return createPortal(
    <div
      className="fixed inset-0 z-[110] overflow-hidden"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
      <div className="w-full h-full flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="relative w-full max-w-2xl bg-[#0a0a0c] border border-white/10 rounded-[2rem] shadow-2xl overflow-hidden flex flex-col h-[90vh]"
        >
          <div className="p-6 border-b border-white/5 flex items-center justify-between bg-white/5">
            <h3 className="text-xl font-bold flex items-center gap-2">
              <Check className="text-primary" />
              Review Mod Selection
            </h3>
            <button onClick={onClose} className="p-2 hover:bg-white/10 rounded-full transition-colors">
              <X size={20} />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar">
            <div>
              <h4 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">Selected Mods</h4>
              <div className="space-y-2">
                {selectedMods.map(mod => (
                  <div
                    key={mod.id}
                    onClick={() => toggleSelection(mod.id)}
                    className={`flex items-center gap-3 p-3 rounded-xl border transition-all cursor-pointer ${selectedForInstall.has(mod.id)
                      ? 'bg-primary/10 border-primary/30'
                      : 'bg-white/5 border-white/5 opacity-50'
                      }`}
                  >
                    <div className={`w-5 h-5 rounded flex items-center justify-center border transition-all ${selectedForInstall.has(mod.id) ? 'bg-primary border-primary' : 'border-white/20'
                      }`}>
                      {selectedForInstall.has(mod.id) && <Check size={14} className="text-white" />}
                    </div>
                    {mod.icon_url ? (
                      <img src={mod.icon_url} alt="" className="w-8 h-8 rounded-lg object-cover" />
                    ) : (
                      <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-gray-500">
                        <Package size={16} />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <span className="font-medium block truncate">{mod.title}</span>
                      <span className={`text-[10px] font-bold uppercase tracking-tight ${mod.provider === 'Modrinth' ? 'text-green-500' : 'text-orange-500'}`}>
                        {mod.provider}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {(dependencies.length > 0) && (
              <div>
                <h4 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3 flex items-center gap-2">
                  Dependencies
                </h4>
                <div className="space-y-2">
                  {dependencies.map(dep => (
                    <div
                      key={dep.id}
                      onClick={() => toggleSelection(dep.id)}
                      className={`flex items-center gap-3 p-3 rounded-xl border transition-all cursor-pointer ${selectedForInstall.has(dep.id)
                        ? 'bg-blue-500/10 border-blue-500/30'
                        : 'bg-white/5 border-white/5 opacity-50'
                        }`}
                    >
                      <div className={`w-5 h-5 rounded flex items-center justify-center border transition-all ${selectedForInstall.has(dep.id) ? 'bg-blue-500 border-blue-500' : 'border-white/20'
                        }`}>
                        {selectedForInstall.has(dep.id) && <Check size={14} className="text-white" />}
                      </div>
                      {dep.icon_url ? (
                        <img src={dep.icon_url} alt="" className="w-8 h-8 rounded-lg object-cover" />
                      ) : (
                        <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-gray-500">
                          <Package size={16} />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <span className="font-medium truncate block">{dep.title}</span>
                        <div className="flex items-center gap-2">
                          <span className="text-[10px] text-blue-400 font-bold uppercase tracking-tight">Required Dependency</span>
                          <span className={`text-[10px] font-bold uppercase tracking-tight ${dep.provider === 'Modrinth' ? 'text-green-500' : 'text-orange-500'}`}>
                            {dep.provider}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="bg-primary/5 border border-primary/10 rounded-2xl p-4 flex gap-3">
              <AlertCircle className="text-primary shrink-0" size={20} />
              <p className="text-xs text-gray-400 leading-relaxed">
                Only mods with a checkmark will be downloaded. Dependencies are automatically selected but you can uncheck them if you already have them installed.
              </p>
            </div>
          </div>

          <div className="p-6 border-t border-white/5 bg-white/5 flex gap-3">
            <button
              onClick={onClose}
              className="flex-1 py-3 bg-white/5 hover:bg-white/10 text-white rounded-xl font-bold transition-all"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={isInstalling || selectedForInstall.size === 0}
              className="flex-[2] py-3 bg-primary text-white rounded-xl font-bold shadow-lg shadow-primary/20 hover:bg-primary/90 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {isInstalling ? (
                <>
                  <RefreshCw size={20} className="animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <Download size={20} />
                  Install {selectedForInstall.size} Mods
                </>
              )}
            </button>
          </div>
        </motion.div>
      </div>
    </div>,
    document.body
  )
}
