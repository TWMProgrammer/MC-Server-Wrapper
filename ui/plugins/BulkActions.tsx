import { motion } from 'framer-motion'
import { CheckSquare, Power, ArrowUpCircle, Trash2 } from 'lucide-react'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface BulkActionsProps {
  selectedCount: number;
  hasUpdates: boolean;
  onBulkToggle: (enable: boolean) => void;
  onBulkUpdate: () => void;
  onBulkDelete: (deleteConfig: boolean) => void;
  onDeselect: () => void;
}

export function BulkActions({
  selectedCount,
  hasUpdates,
  onBulkToggle,
  onBulkUpdate,
  onBulkDelete,
  onDeselect
}: BulkActionsProps) {
  return (
    <motion.div
      initial={{ height: 0, opacity: 0, marginBottom: 0 }}
      animate={{ height: 'auto', opacity: 1, marginBottom: 24 }}
      exit={{ height: 0, opacity: 0, marginBottom: 0 }}
      className="overflow-hidden"
    >
      <div className="bg-primary/10 border border-primary/20 rounded-2xl p-4 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-primary text-white rounded-lg">
            <CheckSquare size={20} />
          </div>
          <div>
            <div className="font-bold text-primary">{selectedCount} Selected</div>
            <div className="text-xs text-primary/60">Bulk actions for selected plugins</div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => onBulkToggle(true)}
            className="flex items-center gap-2 px-3 py-1.5 bg-green-500/20 hover:bg-green-500/30 text-green-500 rounded-lg transition-colors border border-green-500/20 text-sm font-medium"
          >
            <Power size={14} /> Enable
          </button>
          <button
            onClick={() => onBulkToggle(false)}
            className="flex items-center gap-2 px-3 py-1.5 bg-orange-500/20 hover:bg-orange-500/30 text-orange-500 rounded-lg transition-colors border border-orange-500/20 text-sm font-medium"
          >
            <Power size={14} /> Disable
          </button>
          {hasUpdates && (
            <button
              onClick={onBulkUpdate}
              className="flex items-center gap-2 px-3 py-1.5 bg-blue-500/20 hover:bg-blue-500/30 text-blue-500 rounded-lg transition-colors border border-blue-500/20 text-sm font-medium"
            >
              <ArrowUpCircle size={14} /> Update
            </button>
          )}
          <ConfirmDropdown
            title="Uninstall Selected"
            message={`Are you sure you want to uninstall ${selectedCount} plugins?`}
            confirmText="Uninstall All"
            variant="danger"
            onConfirm={() => onBulkDelete(false)}
          >
            <button className="flex items-center gap-2 px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 text-red-500 rounded-lg transition-colors border border-red-500/20 text-sm font-medium">
              <Trash2 size={14} /> Uninstall
            </button>
          </ConfirmDropdown>
          <div className="w-px h-6 bg-primary/20 mx-2" />
          <button
            onClick={onDeselect}
            className="text-primary/60 hover:text-primary text-sm font-medium px-2"
          >
            Deselect
          </button>
        </div>
      </div>
    </motion.div>
  )
}
