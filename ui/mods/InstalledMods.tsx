import { AnimatePresence } from 'framer-motion'
import { ModConfigModal } from './ModConfigModal'
import { InstalledModFilters } from './InstalledModFilters'
import { InstalledModBulkActions } from './InstalledModBulkActions'
import { InstalledModsLoading } from './InstalledModsLoading'
import { InstalledModsEmpty } from './InstalledModsEmpty'
import { InstalledModGridView } from './InstalledModGridView'
import { InstalledModTableView } from './InstalledModTableView'
import { useInstalledMods } from './useInstalledMods'

interface InstalledModsProps {
  instanceId: string;
  refreshTrigger?: number;
}

export function InstalledMods({ instanceId, refreshTrigger }: InstalledModsProps) {
  const {
    loading,
    checkingUpdates,
    searchQuery,
    setSearchQuery,
    viewMode,
    setViewMode,
    selectedFilenames,
    setSelectedFilenames,
    configuringMod,
    setConfiguringMod,
    filteredMods,
    updates,
    updatingMods,
    loadMods,
    handleCheckUpdates,
    handleUpdateMod,
    handleBulkUpdate,
    handleToggleMod,
    handleDeleteMod,
    handleBulkToggle,
    handleBulkDelete,
    toggleSelection,
    toggleAll
  } = useInstalledMods(instanceId, refreshTrigger)

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <InstalledModFilters
          searchQuery={searchQuery}
          setSearchQuery={setSearchQuery}
          viewMode={viewMode}
          setViewMode={setViewMode}
          onCheckUpdates={handleCheckUpdates}
          onRefresh={loadMods}
          loading={loading}
          checkingUpdates={checkingUpdates}
        />

        <AnimatePresence>
          {selectedFilenames.size > 0 && (
            <InstalledModBulkActions
              selectedCount={selectedFilenames.size}
              hasUpdates={updates.some(u => selectedFilenames.has(u.filename))}
              onBulkToggle={handleBulkToggle}
              onBulkUpdate={handleBulkUpdate}
              onBulkDelete={handleBulkDelete}
              onDeselect={() => setSelectedFilenames(new Set())}
            />
          )}
        </AnimatePresence>
      </div>

      {loading ? (
        <InstalledModsLoading />
      ) : filteredMods.length === 0 ? (
        <InstalledModsEmpty searchQuery={searchQuery} />
      ) : viewMode === 'grid' ? (
        <InstalledModGridView
          mods={filteredMods}
          selectedFilenames={selectedFilenames}
          updates={updates}
          updatingMods={updatingMods}
          onUpdate={handleUpdateMod}
          onToggleSelect={toggleSelection}
          onToggleEnabled={handleToggleMod}
          onDelete={handleDeleteMod}
          onConfigure={setConfiguringMod}
        />
      ) : (
        <InstalledModTableView
          mods={filteredMods}
          selectedFilenames={selectedFilenames}
          updates={updates}
          updatingMods={updatingMods}
          onUpdate={handleUpdateMod}
          onToggleSelect={toggleSelection}
          onToggleEnabled={handleToggleMod}
          onDelete={handleDeleteMod}
          onConfigure={setConfiguringMod}
          onToggleAll={toggleAll}
          allSelected={selectedFilenames.size === filteredMods.length && filteredMods.length > 0}
        />
      )}

      <AnimatePresence>
        {configuringMod && (
          <ModConfigModal
            mod={configuringMod}
            instanceId={instanceId}
            onClose={() => setConfiguringMod(null)}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
