import { motion, AnimatePresence } from 'framer-motion'
import { Search, Check, Package, Download, Star, ExternalLink, ChevronRight, Loader2 } from 'lucide-react'
import { cn, formatNumber } from '../utils'
import { Project, ProjectVersion } from '../types'
import { useAssetCache } from '../hooks/useAssetCache'
import { useInView } from '../hooks/useInView'

interface ModpackCardProps {
  project: Project;
  isSelected: boolean;
  onSelect: (project: Project) => void;
}

function ModpackCard({ project, isSelected, onSelect }: ModpackCardProps) {
  const [ref, isInView] = useInView({ rootMargin: '200px' });
  const { localUrl: iconUrl } = useAssetCache(project.icon_url, isInView);

  return (
    <motion.button
      ref={ref as any}
      layout
      whileHover={{ y: -4, scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      onClick={() => onSelect(project)}
      className={cn(
        "group relative flex flex-col gap-4 p-4 rounded-2xl transition-all duration-300 border text-left",
        isSelected
          ? "bg-primary/5 border-primary shadow-glow-primary/10"
          : "bg-black/[0.02] dark:bg-white/[0.02] border-black/5 dark:border-white/5 hover:border-black/10 dark:hover:border-white/10 hover:bg-black/[0.04] dark:hover:bg-white/[0.04]"
      )}
    >
      <div className="flex gap-4">
        <div className="relative shrink-0">
          {project.icon_url ? (
            <img src={iconUrl || project.icon_url} alt="" className="w-16 h-16 rounded-xl object-cover bg-black/20 shadow-lg" />
          ) : (
            <div className="w-16 h-16 rounded-xl bg-primary/10 text-primary flex items-center justify-center shadow-lg">
              <Package size={32} />
            </div>
          )}
          {isSelected && (
            <div className="absolute -top-2 -right-2 bg-primary text-white p-1 rounded-full shadow-lg z-10 border-2 border-white dark:border-gray-950">
              <Check size={10} strokeWidth={4} />
            </div>
          )}
        </div>

        <div className="flex-1 min-w-0">
          <h3 className="font-bold text-gray-900 dark:text-white truncate text-base group-hover:text-primary transition-colors">
            {project.title}
          </h3>
          <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-2 font-medium mt-1">
            {project.description}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-4 pt-2 border-t border-black/5 dark:border-white/5 mt-auto">
        <div className="flex items-center gap-1.5">
          <Download size={12} className="text-primary" />
          <span className="text-[10px] font-bold text-gray-500">{formatNumber(project.downloads)}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Star size={12} className="text-primary" />
          <span className="text-[10px] font-bold text-gray-500">{(project.downloads / 5000).toFixed(0)}</span>
        </div>
        <div className="ml-auto text-[9px] font-black uppercase tracking-widest text-primary/60">
          {project.author}
        </div>
      </div>
    </motion.button>
  );
}

interface ModrinthSourceProps {
  search: string;
  setSearch: (s: string) => void;
  results: Project[];
  searching: boolean;
  selectedModpack: Project | null;
  setSelectedModpack: (p: Project | null) => void;
  versions: ProjectVersion[];
  selectedVersion: string | null;
  setSelectedVersion: (v: string | null) => void;
  loadingVersions: boolean;
}

export function ModrinthSource({
  search,
  setSearch,
  results,
  searching,
  selectedModpack,
  setSelectedModpack,
  versions,
  selectedVersion,
  setSelectedVersion,
  loadingVersions
}: ModrinthSourceProps) {
  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="px-6 py-4 flex items-center justify-between border-b border-black/5 dark:border-white/5 bg-black/[0.01] dark:bg-white/[0.01]">
        <div className="flex items-center gap-3">
          {selectedModpack && (
            <motion.button
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              onClick={() => setSelectedModpack(null)}
              className="p-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl text-gray-400 dark:text-white/50 hover:text-gray-900 dark:hover:text-white transition-all duration-200 border border-black/5 dark:border-white/5"
            >
              <ChevronRight size={18} className="rotate-180" />
            </motion.button>
          )}
          <div>
            <h2 className="text-[11px] font-black text-gray-400 dark:text-white/20 uppercase tracking-[0.2em]">
              {selectedModpack ? 'Select Version' : 'Search Modpacks'}
            </h2>
          </div>
        </div>

        {!selectedModpack && (
          <div className="relative w-72 group">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={14} />
            <input
              type="text"
              placeholder="Search Modrinth..."
              value={search}
              onChange={e => setSearch(e.target.value)}
              className="w-full bg-black/5 dark:bg-white/5 border border-black/10 dark:border-white/10 rounded-xl pl-10 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all font-medium"
            />
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-6 custom-scrollbar">
        <AnimatePresence mode="wait">
          {selectedModpack ? (
            <motion.div
              key="versions"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-6"
            >
              <div className="flex gap-6 items-start">
                <img
                  src={selectedModpack.icon_url}
                  alt=""
                  className="w-24 h-24 rounded-2xl object-cover shadow-2xl border-4 border-white dark:border-gray-900"
                />
                <div className="space-y-2">
                  <h1 className="text-2xl font-black text-gray-900 dark:text-white">{selectedModpack.title}</h1>
                  <p className="text-sm text-gray-500 dark:text-gray-400 max-w-xl font-medium">{selectedModpack.description}</p>
                </div>
              </div>

              <div className="space-y-3">
                <h3 className="text-[10px] font-black text-gray-400 dark:text-white/30 uppercase tracking-[0.2em]">Available Versions</h3>
                {loadingVersions ? (
                  <div className="flex items-center gap-3 text-gray-400 py-4">
                    <Loader2 size={16} className="animate-spin" />
                    <span className="text-sm font-bold">Loading versions...</span>
                  </div>
                ) : (
                  <div className="grid grid-cols-1 gap-2">
                    {versions.map(version => (
                      <button
                        key={version.id}
                        onClick={() => setSelectedVersion(version.id)}
                        className={cn(
                          "flex items-center justify-between p-4 rounded-xl border transition-all duration-200",
                          selectedVersion === version.id
                            ? "bg-primary border-primary text-white shadow-glow-primary"
                            : "bg-black/5 dark:bg-white/5 border-black/5 dark:border-white/5 hover:border-black/10 dark:hover:border-white/10 text-gray-900 dark:text-white"
                        )}
                      >
                        <div className="flex flex-col items-start gap-1">
                          <span className="font-bold">{version.version_number}</span>
                          <div className="flex gap-2">
                            {version.game_versions.slice(0, 3).map(v => (
                              <span key={v} className={cn(
                                "text-[9px] font-black uppercase px-1.5 py-0.5 rounded-md",
                                selectedVersion === version.id ? "bg-white/20" : "bg-black/10 dark:bg-white/10"
                              )}>{v}</span>
                            ))}
                          </div>
                        </div>
                        <div className="flex items-center gap-3">
                          <div className="flex gap-1">
                            {version.loaders.map(l => (
                              <span key={l} className={cn(
                                "text-[9px] font-black uppercase px-1.5 py-0.5 rounded-md",
                                selectedVersion === version.id ? "bg-white/20" : "bg-black/10 dark:bg-white/10"
                              )}>{l}</span>
                            ))}
                          </div>
                          {selectedVersion === version.id && <Check size={16} strokeWidth={3} />}
                        </div>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="search"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
            >
              {searching ? (
                Array.from({ length: 6 }).map((_, i) => (
                  <div key={i} className="h-32 rounded-2xl bg-black/5 dark:bg-white/5 animate-pulse" />
                ))
              ) : results.length > 0 ? (
                results.map(project => (
                  <ModpackCard
                    key={project.id}
                    project={project}
                    isSelected={false}
                    onSelect={setSelectedModpack}
                  />
                ))
              ) : (
                <div className="col-span-full py-20 flex flex-col items-center justify-center text-gray-400 dark:text-white/10 gap-4">
                  <Package size={48} strokeWidth={1} />
                  <p className="text-sm font-bold uppercase tracking-widest">No modpacks found</p>
                </div>
              )}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
