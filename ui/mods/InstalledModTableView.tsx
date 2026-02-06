import { motion, AnimatePresence } from 'framer-motion'
import { CheckSquare, Square } from 'lucide-react'
import { InstalledMod, ModUpdate } from '../types'
import { InstalledModTableRow } from './InstalledModTableRow'

interface InstalledModTableViewProps {
  mods: InstalledMod[];
  selectedFilenames: Set<string>;
  updates: ModUpdate[];
  updatingMods: Set<string>;
  onUpdate: (update: ModUpdate) => Promise<void>;
  onToggleSelect: (filename: string) => void;
  onToggleEnabled: (mod: InstalledMod) => Promise<void>;
  onDelete: (mod: InstalledMod, deleteConfig: boolean) => Promise<void>;
  onConfigure: (mod: InstalledMod) => void;
  onToggleAll: () => void;
  allSelected: boolean;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05
    }
  }
};

export function InstalledModTableView({
  mods,
  selectedFilenames,
  updates,
  updatingMods,
  onUpdate,
  onToggleSelect,
  onToggleEnabled,
  onDelete,
  onConfigure,
  onToggleAll,
  allSelected
}: InstalledModTableViewProps) {
  return (
    <div className="w-full">
      <table className="w-full text-left border-collapse table-fixed">
        <thead>
          <tr className="border-b border-white/5">
            <th className="p-4 w-16 shrink-0">
              <button
                onClick={onToggleAll}
                className={`p-1 rounded-md transition-colors ${allSelected ? 'text-primary' : 'text-gray-600 hover:text-gray-400'}`}
              >
                {allSelected ? <CheckSquare size={18} /> : <Square size={18} />}
              </button>
            </th>
            <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider w-auto">Mod</th>
            <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider hidden sm:table-cell w-32 shrink-0">Status</th>
            <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider hidden md:table-cell w-48 shrink-0">Author</th>
            <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider hidden lg:table-cell w-32 shrink-0">Version</th>
            <th className="p-4 text-xs font-bold text-gray-500 uppercase tracking-wider text-right w-44 shrink-0">Actions</th>
          </tr>
        </thead>
        <motion.tbody
          variants={containerVariants}
          initial="hidden"
          animate="visible"
        >
          <AnimatePresence mode="popLayout">
            {mods.map((mod) => (
              <InstalledModTableRow
                key={mod.filename}
                mod={mod}
                isSelected={selectedFilenames.has(mod.filename)}
                update={updates.find(u => u.filename === mod.filename)}
                isUpdating={updatingMods.has(mod.filename)}
                onUpdate={onUpdate}
                onToggleSelect={() => onToggleSelect(mod.filename)}
                onToggleEnabled={() => onToggleEnabled(mod)}
                onDelete={(delConfig) => onDelete(mod, delConfig)}
                onConfigure={() => onConfigure(mod)}
              />
            ))}
          </AnimatePresence>
        </motion.tbody>
      </table>
    </div>
  )
}
