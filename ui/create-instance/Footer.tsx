import { Info, Plus } from 'lucide-react'
import { motion } from 'framer-motion'
import { cn } from '../utils'

interface FooterProps {
  selectedVersion: string | null;
  name: string;
  creating: boolean;
  loadingModLoaders: boolean;
  onClose: () => void;
  onCreate: () => void;
}

export function Footer({
  selectedVersion,
  name,
  creating,
  loadingModLoaders,
  onClose,
  onCreate
}: FooterProps) {
  const isDisabled = !name || !selectedVersion || creating || loadingModLoaders;

  return (
    <div className="p-6 border-t border-black/5 dark:border-white/5 flex items-center justify-between bg-black/5 dark:bg-black/40 backdrop-blur-xl transition-colors duration-300">
      <div className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20 flex items-center gap-3">
        <Info size={16} className="text-primary" />
        <span>{selectedVersion ? `Ready to install Minecraft ${selectedVersion}` : 'Select a software and version to continue'}</span>
      </div>
      <div className="flex items-center gap-4">
        <button
          onClick={onClose}
          className="px-8 py-3 rounded-xl text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-all"
        >
          Cancel
        </button>
        <motion.button
          whileHover={isDisabled ? {} : { scale: 1.02, translateY: -2 }}
          whileTap={isDisabled ? {} : { scale: 0.98 }}
          onClick={onCreate}
          disabled={isDisabled}
          className={cn(
            "px-10 py-3 rounded-xl text-xs font-black uppercase tracking-widest transition-all duration-200 flex items-center gap-3 shadow-2xl",
            isDisabled
              ? "bg-black/5 dark:bg-white/5 text-gray-400 dark:text-white/10 cursor-not-allowed"
              : "bg-primary hover:bg-primary-hover text-white shadow-glow-primary"
          )}
        >
          {creating ? (
            <><div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" /> Creating...</>
          ) : (
            <><Plus size={18} /> Create Instance</>
          )}
        </motion.button>
      </div>
    </div>
  )
}
