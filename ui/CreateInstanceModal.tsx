import { X, Plus, Box } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { Sidebar } from './create-instance/Sidebar'
import { SoftwareSelection } from './create-instance/SoftwareSelection'
import { VersionSelection } from './create-instance/VersionSelection'
import { Footer } from './create-instance/Footer'
import { useCreateInstance } from './create-instance/useCreateInstance'
import { CreateInstanceModalProps } from './create-instance/types'

export function CreateInstanceModal({ isOpen, onClose, onCreated }: CreateInstanceModalProps) {
  const {
    activeTab,
    setActiveTab,
    selectedServerType,
    setSelectedServerType,
    loading,
    loadingModLoaders,
    search,
    setSearch,
    showSnapshots,
    setShowSnapshots,
    name,
    setName,
    selectedVersion,
    setSelectedVersion,
    modLoaders,
    selectedLoaderVersion,
    setSelectedLoaderVersion,
    creating,
    handleCreate,
    filteredVersions
  } = useCreateInstance(isOpen, onCreated, onClose);

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/80 backdrop-blur-md"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.9, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 20 }}
          className="bg-surface border border-black/10 dark:border-white/10 rounded-3xl shadow-2xl w-full max-w-5xl h-[85vh] flex flex-col overflow-hidden relative z-10 ring-1 ring-black/5 dark:ring-white/5 transition-colors duration-300"
        >
          {/* Header */}
          <div className="p-6 border-b border-black/5 dark:border-white/5 flex items-center justify-between gap-6 bg-black/[0.01] dark:bg-white/[0.02]">
            <div className="flex items-center gap-6 flex-1">
              <div className="w-14 h-14 bg-primary/10 rounded-2xl flex items-center justify-center shadow-glow-primary border border-primary/20">
                <Plus className="text-primary" size={28} />
              </div>
              <div className="flex-1 max-w-md">
                <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1.5 ml-1">New Instance</div>
                <input
                  type="text"
                  placeholder="Enter instance name..."
                  value={name}
                  onChange={e => setName(e.target.value)}
                  className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-5 py-3 text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all font-medium"
                  autoFocus
                />
              </div>
            </div>
            <motion.button
              whileHover={{ scale: 1.1, rotate: 90 }}
              whileTap={{ scale: 0.9 }}
              onClick={onClose}
              className="p-3 hover:bg-black/5 dark:hover:bg-white/5 rounded-2xl transition-colors duration-200 text-gray-400 dark:text-white/30 hover:text-gray-900 dark:hover:text-white"
            >
              <X size={24} />
            </motion.button>
          </div>

          <div className="flex-1 flex overflow-hidden">
            <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />

            <div className="flex-1 flex flex-col overflow-hidden bg-background transition-colors duration-300">
              <AnimatePresence mode="wait">
                {activeTab === 'custom' ? (
                  <motion.div
                    key="custom"
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: -20 }}
                    className="flex-1 flex flex-col overflow-hidden"
                  >
                    {!selectedServerType ? (
                      <SoftwareSelection onSelect={setSelectedServerType} />
                    ) : (
                      <VersionSelection
                        selectedServerType={selectedServerType}
                        onBack={() => setSelectedServerType(null)}
                        search={search}
                        setSearch={setSearch}
                        showSnapshots={showSnapshots}
                        setShowSnapshots={setShowSnapshots}
                        loading={loading}
                        filteredVersions={filteredVersions}
                        selectedVersion={selectedVersion}
                        setSelectedVersion={setSelectedVersion}
                        selectedLoaderVersion={selectedLoaderVersion}
                        setSelectedLoaderVersion={setSelectedLoaderVersion}
                        modLoaders={modLoaders}
                        loadingModLoaders={loadingModLoaders}
                      />
                    )}
                  </motion.div>
                ) : (
                  <motion.div
                    key="soon"
                    initial={{ opacity: 0, scale: 0.95 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    className="flex-1 flex flex-col items-center justify-center text-gray-400 dark:text-white/10 gap-6 p-12 text-center"
                  >
                    <div className="w-24 h-24 rounded-full bg-black/5 dark:bg-white/[0.02] border border-black/5 dark:border-white/5 flex items-center justify-center">
                      <Box size={48} strokeWidth={1} />
                    </div>
                    <div className="max-w-xs space-y-2">
                      <h3 className="text-lg font-black text-gray-400 dark:text-white/30 uppercase tracking-[0.2em]">Coming Soon</h3>
                      <p className="text-sm font-medium leading-relaxed text-gray-500 dark:text-white/40">We're working hard to bring {activeTab === 'import' ? 'ZIP imports' : activeTab} to the wrapper!</p>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>

          <Footer
            selectedVersion={selectedVersion}
            name={name}
            creating={creating}
            loadingModLoaders={loadingModLoaders}
            onClose={onClose}
            onCreate={handleCreate}
          />
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
