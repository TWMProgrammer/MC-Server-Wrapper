import { motion } from 'framer-motion'
import { Search, X, RefreshCw, Edit3, Save } from 'lucide-react'
import { ConfigFile } from './types'

interface ConfigControlsProps {
  selectedConfig: ConfigFile | null
  searchTerm: string
  setSearchTerm: (term: string) => void
  onRefresh: () => void
  onRawEdit: () => void
  onSave: () => void
  saving: boolean
}

export function ConfigControls({
  selectedConfig,
  searchTerm,
  setSearchTerm,
  onRefresh,
  onRawEdit,
  onSave,
  saving
}: ConfigControlsProps) {
  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
      <div className="relative flex-1 w-full max-w-xl group">
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={20} />
        <input
          type="text"
          placeholder={`Search in ${selectedConfig?.name || 'properties'}...`}
          className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-2xl py-3.5 pl-12 pr-12 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        {searchTerm && (
          <button
            onClick={() => setSearchTerm('')}
            className="absolute right-4 top-1/2 -translate-y-1/2 p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-all"
          >
            <X size={16} />
          </button>
        )}
      </div>
      <div className="flex gap-3 w-full md:w-auto">
        <motion.button
          whileHover={{
            scale: 1.02,
            translateY: -2,
            transition: { duration: 0.2, ease: "easeOut" }
          }}
          whileTap={{ scale: 0.98 }}
          onClick={onRefresh}
          className="flex-1 md:flex-none flex items-center justify-center gap-2 px-6 py-3.5 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] border border-black/10 dark:border-white/10 rounded-2xl transition-all duration-200 text-sm font-bold uppercase tracking-widest text-gray-500 dark:text-white/60 hover:text-gray-900 dark:hover:text-white"
        >
          <RefreshCw size={18} />
          Refresh
        </motion.button>
        <motion.button
          whileHover={{
            scale: 1.02,
            translateY: -2,
            transition: { duration: 0.2, ease: "easeOut" }
          }}
          whileTap={{ scale: 0.98 }}
          onClick={onRawEdit}
          className="flex-1 md:flex-none flex items-center justify-center gap-2 px-6 py-3.5 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] border border-black/10 dark:border-white/10 rounded-2xl transition-all duration-200 text-sm font-bold uppercase tracking-widest text-gray-500 dark:text-white/60 hover:text-gray-900 dark:hover:text-white"
        >
          <Edit3 size={18} />
          Edit Raw
        </motion.button>
        <motion.button
          whileHover={{
            scale: 1.02,
            translateY: -2,
            transition: { duration: 0.2, ease: "easeOut" }
          }}
          whileTap={{ scale: 0.98 }}
          onClick={onSave}
          disabled={saving}
          className="flex-1 md:flex-none flex items-center justify-center gap-2 px-8 py-3.5 bg-primary hover:bg-primary-hover disabled:bg-black/5 dark:disabled:bg-white/5 disabled:text-gray-400 dark:disabled:text-white/20 disabled:cursor-not-allowed rounded-2xl transition-all duration-200 text-sm font-bold uppercase tracking-widest text-white shadow-glow-primary"
        >
          {saving ? (
            <RefreshCw className="animate-spin" size={18} />
          ) : (
            <Save size={18} />
          )}
          {saving ? 'Saving...' : 'Save Changes'}
        </motion.button>
      </div>
    </div>
  )
}
