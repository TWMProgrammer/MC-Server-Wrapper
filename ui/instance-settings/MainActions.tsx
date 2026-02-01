import { motion } from 'framer-motion'
import { Copy, Trash2 } from 'lucide-react'

interface MainActionsProps {
  onShowClone: () => void;
  onShowDelete: () => void;
}

export function MainActions({ onShowClone, onShowDelete }: MainActionsProps) {
  return (
    <motion.div
      key="main"
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="p-2 space-y-1"
    >
      <div className="px-3 py-2 mb-1">
        <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/50">Instance Actions</h4>
      </div>
      <button
        onClick={onShowClone}
        className="w-full flex items-center gap-3 px-3 py-3 text-sm text-gray-700 dark:text-white/70 hover:bg-black/5 dark:hover:bg-white/[0.05] hover:text-gray-900 dark:hover:text-white rounded-xl transition-all group"
      >
        <div className="w-8 h-8 rounded-lg bg-black/[0.03] dark:bg-white/[0.03] flex items-center justify-center group-hover:bg-primary/20 group-hover:text-primary transition-all">
          <Copy size={16} />
        </div>
        <div className="flex flex-col items-start">
          <span className="font-bold">Clone Instance</span>
          <span className="text-[10px] text-gray-400 dark:text-white/40 uppercase font-black tracking-widest">Duplicate this server</span>
        </div>
      </button>
      <button
        onClick={onShowDelete}
        className="w-full flex items-center gap-3 px-3 py-3 text-sm text-gray-700 dark:text-white/70 hover:bg-accent-rose/10 hover:text-accent-rose rounded-xl transition-all group"
      >
        <div className="w-8 h-8 rounded-lg bg-black/[0.03] dark:bg-white/[0.03] flex items-center justify-center group-hover:bg-accent-rose/20 group-hover:text-accent-rose transition-all">
          <Trash2 size={16} />
        </div>
        <div className="flex flex-col items-start">
          <span className="font-bold">Delete Instance</span>
          <span className="text-[10px] text-gray-400 dark:text-white/40 uppercase font-black tracking-widest">Permanent removal</span>
        </div>
      </button>
    </motion.div>
  );
}
