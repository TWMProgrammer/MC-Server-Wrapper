import { X, Plus, Box, AlertTriangle } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { Sidebar } from './create-instance/Sidebar'
import { SoftwareSelection } from './create-instance/SoftwareSelection'
import { VersionSelection } from './create-instance/VersionSelection'
import { ImportSource } from './create-instance/ImportSource'
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
    error,
    setError,
    handleCreate,
    filteredVersions,
    importSourcePath,
    setImportSourcePath,
    importServerType,
    setImportServerType,
    availableJars,
    setAvailableJars,
    selectedJar,
    setSelectedJar,
    serverPropertiesExists,
    setServerPropertiesExists,
    rootWithinZip,
    setRootWithinZip,
    importProgress,
    startAfterCreation,
    setStartAfterCreation,
    nameExists
  } = useCreateInstance(isOpen, onCreated, onClose);

  if (!isOpen) return null;

  const percentage = importProgress?.total ? Math.round((importProgress.current / importProgress.total) * 100) : 0;

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
          className="bg-white dark:bg-gray-950 border border-black/10 dark:border-white/10 rounded-3xl shadow-2xl w-[90vw] max-w-6xl h-[85vh] max-h-[850px] flex flex-col overflow-hidden relative z-10 ring-1 ring-black/5 dark:ring-white/5 transition-colors duration-300"
        >
          {/* Progress Overlay */}
          <AnimatePresence>
            {creating && activeTab === 'import' && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="absolute inset-0 z-[60] bg-white/95 dark:bg-gray-950/95 backdrop-blur-md flex items-center justify-center p-8"
              >
                <div className="w-full max-w-lg space-y-12">
                  <div className="flex flex-col items-center text-center gap-6">
                    <div className="w-20 h-20 bg-primary/10 rounded-[2rem] flex items-center justify-center shadow-glow-primary border border-primary/20 relative group">
                      <div className="absolute inset-0 bg-primary/5 rounded-[2rem] animate-ping opacity-20" />
                      <Box className="text-primary relative z-10" size={40} />
                    </div>
                    <div className="space-y-2">
                      <div className="text-[10px] font-black uppercase tracking-[0.3em] text-primary/60">Importing Instance</div>
                      <h3 className="text-3xl font-black text-gray-900 dark:text-white tracking-tight">Processing Files</h3>
                    </div>
                  </div>

                  <div className="space-y-6 bg-black/5 dark:bg-white/[0.02] p-8 rounded-[2.5rem] border border-black/5 dark:border-white/5 shadow-inner">
                    <div className="flex justify-between items-end gap-8">
                      <div className="space-y-2 flex-1 min-w-0">
                        <span className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20 ml-1">Current Task</span>
                        <div className="text-sm font-bold text-gray-700 dark:text-white/80 flex items-center gap-3">
                          <div className="w-2 h-2 rounded-full bg-primary animate-pulse flex-shrink-0" />
                          <span className="truncate block" title={importProgress?.message}>
                            {importProgress?.message || 'Starting import...'}
                          </span>
                        </div>
                      </div>
                      <div className="text-4xl font-black font-mono tracking-tighter text-primary tabular-nums">
                        {percentage}<span className="text-lg ml-1 opacity-50">%</span>
                      </div>
                    </div>

                    <div className="space-y-3">
                      <div className="h-4 bg-black/10 dark:bg-white/5 rounded-full overflow-hidden border border-black/5 dark:border-white/5 p-1 relative">
                        <motion.div
                          initial={{ width: 0 }}
                          animate={{ width: `${percentage}%` }}
                          transition={{ type: "spring", bounce: 0, duration: 0.5 }}
                          className="h-full bg-gradient-to-r from-primary via-primary-light to-primary rounded-full shadow-glow-primary relative min-w-[1rem]"
                        >
                          <div className="absolute inset-0 bg-[linear-gradient(45deg,rgba(255,255,255,0.2)_25%,transparent_25%,transparent_50%,rgba(255,255,255,0.2)_50%,rgba(255,255,255,0.2)_75%,transparent_75%,transparent)] bg-[length:24px_24px] animate-[progress-stripe_1s_linear_infinite]" />
                        </motion.div>
                      </div>

                      <div className="flex justify-between items-center px-1">
                        <div className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20">
                          <span className="text-primary/40">{importProgress?.current || 0}</span>
                          <span className="opacity-30">/</span>
                          <span>{importProgress?.total || 0} items</span>
                        </div>
                        <div className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20">
                          {percentage < 100 ? 'In Progress' : 'Finishing up...'}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Error Message */}
          <AnimatePresence>
            {error && (
              <motion.div
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
                className="absolute top-20 left-1/2 -translate-x-1/2 z-[80] bg-accent-rose/10 border border-accent-rose/20 text-accent-rose px-6 py-3 rounded-2xl flex items-center gap-3 shadow-glow-rose backdrop-blur-md"
              >
                <AlertTriangle size={18} />
                <span className="text-xs font-bold">{error}</span>
                <button
                  onClick={() => setError(null)}
                  className="ml-2 hover:opacity-70 transition-opacity"
                >
                  <X size={14} />
                </button>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Header */}
          <div className="p-4 border-b border-black/5 dark:border-white/5 flex items-center justify-between gap-6 bg-black/[0.01] dark:bg-white/[0.02]">
            <div className="flex items-center gap-4 flex-1">
              <div className="w-12 h-12 bg-primary/10 rounded-xl flex items-center justify-center shadow-glow-primary border border-primary/20">
                <Plus className="text-primary" size={24} />
              </div>
              <div className="flex-1 max-w-md relative">
                <div className="text-[9px] font-black uppercase tracking-[0.2em] text-primary mb-1 ml-1">New Instance</div>
                <input
                  type="text"
                  placeholder="Enter instance name..."
                  value={name}
                  onChange={e => setName(e.target.value)}
                  className={`w-full bg-black/5 dark:bg-white/[0.03] border rounded-xl px-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 transition-all font-medium ${nameExists
                    ? 'border-accent-rose focus:ring-accent-rose/50 focus:border-accent-rose/50'
                    : 'border-black/10 dark:border-white/10 focus:ring-primary/50 focus:border-primary/50'
                    }`}
                  autoFocus
                />
                <AnimatePresence>
                  {nameExists && (
                    <motion.div
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -10 }}
                      className="absolute -bottom-5 left-1 text-[10px] font-bold text-accent-rose flex items-center gap-1"
                    >
                      <AlertTriangle size={10} />
                      An instance with this name already exists
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </div>
            <motion.button
              whileHover={{ scale: 1.1, rotate: 90 }}
              whileTap={{ scale: 0.9 }}
              onClick={onClose}
              className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-xl transition-colors duration-200 text-gray-400 dark:text-white/30 hover:text-gray-900 dark:hover:text-white"
            >
              <X size={20} />
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
                ) : activeTab === 'import' ? (
                  <motion.div
                    key="import"
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: -20 }}
                    className="flex-1 flex flex-col overflow-hidden"
                  >
                    <ImportSource
                      importSourcePath={importSourcePath}
                      setImportSourcePath={setImportSourcePath}
                      importServerType={importServerType}
                      setImportServerType={setImportServerType}
                      availableJars={availableJars}
                      setAvailableJars={setAvailableJars}
                      selectedJar={selectedJar}
                      setSelectedJar={setSelectedJar}
                      serverPropertiesExists={serverPropertiesExists}
                      setServerPropertiesExists={setServerPropertiesExists}
                      rootWithinZip={rootWithinZip}
                      setRootWithinZip={setRootWithinZip}
                    />
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
                      <p className="text-sm font-medium leading-relaxed text-gray-500 dark:text-white/40">We're working hard to bring {activeTab} to the wrapper!</p>
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
            activeTab={activeTab}
            importSourcePath={importSourcePath}
            selectedJar={selectedJar}
            serverPropertiesExists={serverPropertiesExists}
            startAfterCreation={startAfterCreation}
            setStartAfterCreation={setStartAfterCreation}
            nameExists={nameExists}
          />
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
