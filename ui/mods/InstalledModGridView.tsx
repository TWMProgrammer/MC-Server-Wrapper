import { InstalledMod, ModUpdate } from '../types'
import { InstalledModCard } from './InstalledModCard'

interface InstalledModGridViewProps {
  mods: InstalledMod[];
  selectedFilenames: Set<string>;
  updates: ModUpdate[];
  updatingMods: Set<string>;
  onUpdate: (update: ModUpdate) => Promise<void>;
  onToggleSelect: (filename: string) => void;
  onToggleEnabled: (mod: InstalledMod) => Promise<void>;
  onDelete: (mod: InstalledMod, deleteConfig: boolean) => Promise<void>;
  onConfigure: (mod: InstalledMod) => void;
}

export function InstalledModGridView({
  mods,
  selectedFilenames,
  updates,
  updatingMods,
  onUpdate,
  onToggleSelect,
  onToggleEnabled,
  onDelete,
  onConfigure
}: InstalledModGridViewProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      {mods.map((mod) => (
        <InstalledModCard
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
    </div>
  )
}
