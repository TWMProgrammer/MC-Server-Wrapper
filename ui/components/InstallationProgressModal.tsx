import { motion, AnimatePresence } from 'framer-motion'
import { createPortal } from 'react-dom'
import { Download, Loader2, CheckCircle2, Package } from 'lucide-react'
import { useAppSettings } from '../hooks/useAppSettings'
import { cn } from '../utils'

interface InstallationProgressModalProps {
  isOpen: boolean;
  currentCount: number;
  totalCount: number;
  currentName: string;
  type: 'mod' | 'plugin';
}

export function InstallationProgressModal({
  isOpen,
  currentCount,
  totalCount,
  currentName,
  type
}: InstallationProgressModalProps) {
  const { settings } = useAppSettings()
  
  if (!isOpen) return null

  const percentage = Math.round((currentCount / totalCount) * 100)
  const isFinished = currentCount === totalCount && totalCount > 0

  return createPortal(
    <div
      className="fixed inset-0 z-[200] overflow-hidden"
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
          className="absolute inset-0 bg-black/80 backdrop-blur-xl"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="relative w-full max-w-md bg-[#0a0a0c] border border-white/10 rounded-[2.5rem] shadow-2xl overflow-hidden p-8"
        >
          <div className="flex items-center gap-6 mb-8">
            <div className={cn(
              "w-16 h-16 rounded-2xl flex items-center justify-center transition-all duration-500 shadow-2xl",
              isFinished ? "bg-emerald-500/20 text-emerald-400 rotate-[360deg]" : "bg-primary/20 text-primary animate-pulse"
            )}>
              {isFinished ? <CheckCircle2 size={32} /> : <Download size={32} />}
            </div>
            <div>
              <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1">
                Marketplace
              </div>
              <h3 className="text-xl font-black text-white">
                {isFinished ? 'Installation Complete!' : `Installing ${type === 'mod' ? 'Mods' : 'Plugins'}`}
              </h3>
              <p className="text-sm font-medium text-gray-500 truncate max-w-[200px]">
                {isFinished ? 'All set!' : `Installing ${currentCount + 1} of ${totalCount}`}
              </p>
            </div>
          </div>

          <div className="space-y-6">
            <div className="flex justify-between items-end">
              <div className="space-y-1 flex-1 min-w-0">
                <span className="text-[10px] font-black uppercase tracking-widest text-white/20">Current {type === 'mod' ? 'Mod' : 'Plugin'}</span>
                <div className="text-sm font-bold text-white/80 flex items-center gap-2">
                  {!isFinished && <Loader2 size={14} className="animate-spin text-primary shrink-0" />}
                  <span className="truncate">{isFinished ? 'Finished' : currentName}</span>
                </div>
              </div>
              <div className="text-2xl font-black font-mono tracking-tighter text-primary ml-4">
                {percentage}%
              </div>
            </div>

            <div className="h-3 w-full bg-white/5 rounded-full overflow-hidden p-0.5 border border-white/5">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${percentage}%` }}
                className={cn(
                  "h-full rounded-full transition-all duration-300 shadow-glow-primary relative overflow-hidden",
                  isFinished ? "bg-emerald-500 shadow-glow-emerald" : "bg-primary"
                )}
              >
                <motion.div
                  animate={{ x: ['-100%', '100%'] }}
                  transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                  className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent"
                />
              </motion.div>
            </div>

            <div className="flex items-center gap-3 bg-white/[0.02] p-4 rounded-2xl border border-white/5">
              <div className="w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center text-gray-500">
                <Package size={20} />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-[10px] font-black uppercase tracking-widest text-white/20">Progress</div>
                <div className="text-xs font-bold text-gray-400">
                  {currentCount} / {totalCount} {type === 'mod' ? 'mods' : 'plugins'} processed
                </div>
              </div>
            </div>
          </div>
          
          <div className="mt-8 text-center">
            <p className="text-[10px] font-black uppercase tracking-[0.2em] text-white/20">
              Please do not close the application
            </p>
          </div>
        </motion.div>
      </div>
    </div>,
    document.body
  )
}
