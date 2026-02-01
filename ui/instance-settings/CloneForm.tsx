import { motion } from 'framer-motion'
import { ChevronLeft, RefreshCw, Copy } from 'lucide-react'

interface CloneFormProps {
  cloneName: string;
  setCloneName: (name: string) => void;
  onClone: () => void;
  onBack: () => void;
  isCloning: boolean;
}

export function CloneForm({ cloneName, setCloneName, onClone, onBack, isCloning }: CloneFormProps) {
  return (
    <motion.div
      key="clone"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      className="p-4 space-y-4"
    >
      <div className="flex items-center gap-3 mb-2">
        <button
          onClick={onBack}
          className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-colors"
        >
          <ChevronLeft size={16} />
        </button>
        <span className="text-xs font-black uppercase tracking-widest text-gray-500 dark:text-white/60">Clone Server</span>
      </div>

      <div className="space-y-2">
        <label className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20 ml-1">New Instance Name</label>
        <input
          type="text"
          value={cloneName}
          onChange={e => setCloneName(e.target.value)}
          className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-4 py-3 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all"
          placeholder="New name..."
          autoFocus
        />
      </div>

      <div className="flex gap-2 pt-2">
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={onClone}
          disabled={isCloning || !cloneName.trim()}
          className="flex-1 py-3 bg-primary hover:bg-primary-hover disabled:opacity-50 text-white rounded-xl text-xs font-black uppercase tracking-widest shadow-glow-primary transition-all flex items-center justify-center gap-2"
        >
          {isCloning ? <RefreshCw size={14} className="animate-spin" /> : <Copy size={14} />}
          {isCloning ? 'Cloning...' : 'Clone'}
        </motion.button>
      </div>
    </motion.div>
  );
}
