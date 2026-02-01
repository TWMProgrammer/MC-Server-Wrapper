import { motion } from 'framer-motion'
import { ChevronRight, Search, Check, Terminal } from 'lucide-react'
import { cn } from '../utils'
import { MCVersion, ModLoader } from './types'
import { SERVER_TYPES } from './constants'
import { Select } from '../components/Select'

interface VersionSelectionProps {
  selectedServerType: string;
  onBack: () => void;
  search: string;
  setSearch: (s: string) => void;
  showSnapshots: boolean;
  setShowSnapshots: (s: boolean) => void;
  loading: boolean;
  filteredVersions: MCVersion[];
  selectedVersion: string | null;
  setSelectedVersion: (v: string) => void;
  selectedLoaderVersion: string | null;
  setSelectedLoaderVersion: (v: string) => void;
  modLoaders: ModLoader[];
  loadingModLoaders: boolean;
}

export function VersionSelection({
  selectedServerType,
  onBack,
  search,
  setSearch,
  showSnapshots,
  setShowSnapshots,
  loading,
  filteredVersions,
  selectedVersion,
  setSelectedVersion,
  selectedLoaderVersion,
  setSelectedLoaderVersion,
  modLoaders,
  loadingModLoaders
}: VersionSelectionProps) {
  const selectedServer = SERVER_TYPES.find(t => t.id === selectedServerType);

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="p-6 border-b border-black/5 dark:border-white/5 flex items-center justify-between bg-black/[0.01] dark:bg-white/[0.01]">
        <div className="flex items-center gap-4">
          <motion.button
            whileHover={{
              scale: 1.1,
              x: -2,
              transition: { duration: 0.2, ease: "easeOut" }
            }}
            whileTap={{ scale: 0.9 }}
            onClick={onBack}
            className="p-2.5 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl text-gray-400 dark:text-white/50 hover:text-gray-900 dark:hover:text-white transition-all duration-200 border border-black/5 dark:border-white/5"
          >
            <ChevronRight size={20} className="rotate-180" />
          </motion.button>
          <div>
            <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1">Software</div>
            <h3 className="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2">
              {selectedServer?.name}
            </h3>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <div className="relative w-72 group">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={16} />
            <input
              type="text"
              placeholder="Search versions..."
              value={search}
              onChange={e => setSearch(e.target.value)}
              className="w-full bg-black/5 dark:bg-white/5 border border-black/10 dark:border-white/10 rounded-xl pl-12 pr-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all font-medium"
            />
          </div>
          <div className="flex items-center gap-4 text-[11px] font-bold text-gray-500 dark:text-white/40">
            <label className="flex items-center gap-2.5 cursor-pointer hover:text-gray-900 dark:hover:text-white transition-colors group">
              <div className={cn(
                "w-5 h-5 rounded-md border-2 flex items-center justify-center transition-all",
                showSnapshots ? "bg-primary border-primary shadow-glow-primary" : "border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5"
              )}>
                {showSnapshots && <Check size={12} className="text-white" />}
              </div>
              <input
                type="checkbox"
                checked={showSnapshots}
                onChange={e => setShowSnapshots(e.target.checked)}
                className="hidden"
              />
              Snapshots
            </label>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto custom-scrollbar">
        {loading ? (
          <div className="h-full flex flex-col items-center justify-center gap-4 text-gray-400 dark:text-white/20">
            <div className="w-12 h-12 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
            <span className="text-sm font-bold tracking-widest uppercase">Fetching versions...</span>
          </div>
        ) : (
          <table className="w-full text-left text-sm border-separate border-spacing-y-2 px-6">
            <thead className="sticky top-0 bg-background/80 backdrop-blur-md z-10 transition-colors duration-300">
              <tr className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">
                <th className="px-6 py-4">Version</th>
                <th className="px-6 py-4">Type</th>
                <th className="px-6 py-4">Release Date</th>
              </tr>
            </thead>
            <tbody>
              {filteredVersions.map(v => (
                <motion.tr
                  key={v.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  onClick={() => setSelectedVersion(v.id)}
                  className={cn(
                    "cursor-pointer transition-all group",
                    selectedVersion === v.id
                      ? "bg-primary/10 text-gray-900 dark:text-white"
                      : "text-gray-600 dark:text-white/50 hover:bg-black/[0.02] dark:hover:bg-white/[0.03]"
                  )}
                >
                  <td className="px-6 py-4 first:rounded-l-2xl border-y border-l border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                    <div className="flex items-center gap-3">
                      <div className={cn(
                        "w-6 h-6 rounded-full flex items-center justify-center transition-all",
                        selectedVersion === v.id ? "bg-primary text-white scale-110 shadow-glow-primary" : "bg-black/5 dark:bg-white/5 text-transparent border border-black/10 dark:border-white/10"
                      )}>
                        <Check size={14} />
                      </div>
                      <span className="font-bold font-mono tracking-tight">{v.id}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4 border-y border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                    <span className={cn(
                      "px-2.5 py-1 rounded-lg text-[10px] font-black uppercase tracking-wider",
                      v.type === 'release' ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400" : "bg-amber-500/10 text-amber-600 dark:text-amber-400"
                    )}>
                      {v.type.replace('_', ' ')}
                    </span>
                  </td>
                  <td className="px-6 py-4 last:rounded-r-2xl border-y border-r border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                    <span className="text-gray-400 dark:text-white/20 font-medium group-hover:text-gray-900 dark:group-hover:text-white/40 transition-colors">
                      {new Date(v.releaseTime).toLocaleDateString(undefined, { dateStyle: 'long' })}
                    </span>
                  </td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Mod Loader Selection */}
      {['forge', 'fabric', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '') && (
        <motion.div
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          className="p-6 border-t border-black/5 dark:border-white/5 bg-black/[0.01] dark:bg-white/[0.02] flex items-center justify-between transition-colors duration-300"
        >
          <div className="flex items-center gap-4">
            <div className="p-3 rounded-xl bg-primary/10 text-primary">
              <Terminal size={20} />
            </div>
            <div>
              <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-0.5">Software Build</div>
              <div className="text-sm font-bold text-gray-600 dark:text-white/70">
                Select {selectedServer?.name} version
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3 min-w-[200px]">
            <Select
              value={selectedLoaderVersion || ''}
              onChange={newValue => setSelectedLoaderVersion(newValue)}
              options={
                modLoaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase()))?.versions.map(v => ({
                  value: v,
                  label: v
                })) || []
              }
              placeholder="Select version"
              direction="up"
              loading={loadingModLoaders}
            />
          </div>
        </motion.div>
      )}
    </div>
  );
}
