import { motion } from 'framer-motion'
import { AlertTriangle, X } from 'lucide-react'

interface DeleteConfirmProps {
  instanceName: string;
  onDelete: () => void;
  onCancel: () => void;
  isDeleting: boolean;
}

export function DeleteConfirm({ instanceName, onDelete, onCancel, isDeleting }: DeleteConfirmProps) {
  return (
    <motion.div
      key="delete"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      className="p-4 space-y-4"
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2 text-accent-rose">
          <AlertTriangle size={18} />
          <span className="text-xs font-black uppercase tracking-widest">Dangerous</span>
        </div>
        <button
          onClick={onCancel}
          className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <div className="space-y-2">
        <p className="text-xs text-gray-600 dark:text-white/60 leading-relaxed">
          Are you sure you want to delete <strong className="text-gray-900 dark:text-white font-bold">{instanceName}</strong>? This action cannot be undone.
        </p>
      </div>

      <div className="flex flex-col gap-2 pt-2">
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={onDelete}
          disabled={isDeleting}
          className="w-full py-3 bg-accent-rose hover:bg-accent-rose/80 disabled:opacity-50 text-white rounded-xl text-xs font-black uppercase tracking-widest shadow-glow-rose transition-all"
        >
          {isDeleting ? 'Deleting...' : 'Permanently Delete'}
        </motion.button>
        <button
          onClick={onCancel}
          className="w-full py-3 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] text-gray-500 dark:text-white/60 hover:text-gray-900 dark:hover:text-white rounded-xl text-[10px] font-black uppercase tracking-[0.2em] transition-all"
        >
          Keep Instance
        </button>
      </div>
    </motion.div>
  );
}
