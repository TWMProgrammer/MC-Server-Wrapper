import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Search,
  Download,
  ExternalLink,
  Star,
  User,
  Package,
  Filter,
  RefreshCw,
  Globe,
  ChevronRight,
  ChevronLeft,
  Check,
  Tag,
  Layers,
  Cpu
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { Project, ModProvider, SortOrder, SearchOptions, Instance } from '../types'
import { useToast } from '../hooks/useToast'
import { ModDetailsModal } from './ModDetailsModal'
import { ModReviewModal } from './ModReviewModal'
import { Select } from '../components/Select'

interface ModMarketplaceProps {
  instanceId: string;
  onInstallSuccess?: () => void;
}

const CATEGORIES = [
  { id: 'adventure', name: 'Adventure', icon: '🗺️' },
  { id: 'decoration', name: 'Decoration', icon: '🎨' },
  { id: 'equipment', name: 'Equipment', icon: '⚔️' },
  { id: 'food', name: 'Food', icon: '🍕' },
  { id: 'library', name: 'Library', icon: '📚' },
  { id: 'magic', name: 'Magic', icon: '🧙' },
  { id: 'management', name: 'Management', icon: '📋' },
  { id: 'optimization', name: 'Optimization', icon: '⚡' },
  { id: 'storage', name: 'Storage', icon: '📦' },
  { id: 'technology', name: 'Technology', icon: '⚙️' },
  { id: 'utility', name: 'Utility', icon: '🛠️' },
  { id: 'world-gen', name: 'World Gen', icon: '🌍' },
]

const SORT_OPTIONS = [
  { value: 'Relevance', label: 'Sort by Relevance' },
  { value: 'Downloads', label: 'Sort by Downloads' },
  { value: 'Follows', label: 'Sort by Follows' },
  { value: 'Newest', label: 'Sort by Newest' },
  { value: 'Updated', label: 'Sort by Updated' },
]

export function ModMarketplace({ instanceId, onInstallSuccess }: ModMarketplaceProps) {
  const [query, setQuery] = useState('')
  const [provider, setProvider] = useState<ModProvider>('Modrinth')
  const [results, setResults] = useState<Project[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [selectedMods, setSelectedMods] = useState<Map<string, Project>>(new Map())
  const [showReview, setShowReview] = useState(false)
  const [isInstalling, setIsInstalling] = useState(false)
  const [isResolvingDeps, setIsResolvingDeps] = useState(false)
  const [resolvedDeps, setResolvedDeps] = useState<Project[]>([])
  const [activeCategory, setActiveCategory] = useState<string | null>(null)
  const [sortOrder, setSortOrder] = useState<SortOrder>('Downloads')
  const [page, setPage] = useState(1)
  const [instance, setInstance] = useState<Instance | null>(null)
  const PAGE_SIZE = 16

  const { showToast } = useToast()

  // Load instance details to get default version and loader
  useEffect(() => {
    const loadInstance = async () => {
      try {
        const instances = await invoke<Instance[]>('list_instances')
        const current = instances.find(i => i.id === instanceId)
        if (current) {
          setInstance(current)
        }
      } catch (err) {
        console.error('Failed to load instance:', err)
      }
    }
    loadInstance()
  }, [instanceId])

  const handleSearch = async (e?: React.FormEvent) => {
    e?.preventDefault()
    setLoading(true)
    try {
      const facets: string[] = []

      if (activeCategory) {
        facets.push(`categories:${activeCategory}`)
      }

      const searchOptions: SearchOptions = {
        query: query.trim(),
        facets: facets.length > 0 ? facets : undefined,
        sort: sortOrder,
        offset: (page - 1) * PAGE_SIZE,
        limit: PAGE_SIZE,
        game_version: instance?.version,
        loader: instance?.mod_loader,
      }

      const searchResults = await invoke<Project[]>('search_mods', {
        options: searchOptions,
        provider
      })
      setResults(searchResults)
    } catch (err) {
      console.error('Search failed:', err)
      showToast('Search failed: ' + err, 'error')
    } finally {
      setLoading(false)
    }
  }

  // Initial search on load or when instance/filters change
  useEffect(() => {
    handleSearch()
  }, [provider, activeCategory, sortOrder, page, instance])

  // Reset page when filters change
  useEffect(() => {
    setPage(1)
  }, [provider, activeCategory, sortOrder, query])

  const toggleModSelection = (project: Project) => {
    const newSelection = new Map(selectedMods)
    if (newSelection.has(project.id)) {
      newSelection.delete(project.id)
    } else {
      newSelection.set(project.id, project)
    }
    setSelectedMods(newSelection)
  }

  const handleReview = async () => {
    setIsResolvingDeps(true)
    const allDeps: Map<string, Project> = new Map()
    const seenIds = new Set(Array.from(selectedMods.keys()))
    const queue = Array.from(selectedMods.values())

    try {
      while (queue.length > 0) {
        const mod = queue.shift()!

        const deps = await invoke<Project[]>('get_mod_dependencies', {
          projectId: mod.id,
          provider: mod.provider
        })

        for (const dep of deps) {
          if (!seenIds.has(dep.id)) {
            allDeps.set(dep.id, dep)
            seenIds.add(dep.id)
            queue.push(dep)
          }
        }
      }
      setResolvedDeps(Array.from(allDeps.values()))
      setShowReview(true)
    } catch (err) {
      console.error('Failed to fetch dependencies:', err)
      showToast('Failed to resolve dependencies: ' + err, 'error')
      setShowReview(true)
    } finally {
      setIsResolvingDeps(false)
    }
  }

  const handleConfirmInstall = async (mods: Project[]) => {
    setIsInstalling(true)
    try {
      for (const mod of mods) {
        await invoke('install_mod', {
          instanceId,
          projectId: mod.id,
          provider: mod.provider,
          versionId: null // Latest compatible
        })
      }
      showToast(`Successfully installed ${mods.length} mods!`, 'success')
      setSelectedMods(new Map())
      setShowReview(false)
      onInstallSuccess?.()
    } catch (err) {
      console.error('Installation failed:', err)
      showToast('Installation failed: ' + err, 'error')
    } finally {
      setIsInstalling(false)
    }
  }

  return (
    <div className="flex flex-1 gap-8 overflow-hidden min-h-0">
      {/* Sidebar */}
      <div className="w-64 flex flex-col gap-6 shrink-0 overflow-y-auto custom-scrollbar pr-2">
        <div className="space-y-4">
          <div className="flex items-center gap-2 text-sm font-bold text-gray-400 uppercase tracking-widest px-2">
            <Globe size={16} />
            Providers
          </div>
          <div className="space-y-1">
            {(['Modrinth', 'CurseForge'] as const).map((p) => (
              <button
                key={p}
                onClick={() => setProvider(p)}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-2xl text-sm font-bold transition-all ${provider === p
                  ? 'bg-primary text-white shadow-lg shadow-primary/20'
                  : 'text-gray-500 hover:text-gray-300 hover:bg-white/5'
                  }`}
              >
                <div className={`w-2 h-2 rounded-full ${p === 'Modrinth' ? 'bg-green-500' : 'bg-orange-500'}`} />
                {p}
              </button>
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex items-center gap-2 text-sm font-bold text-gray-400 uppercase tracking-widest px-2">
            <Tag size={16} />
            Categories
          </div>
          <div className="space-y-1">
            <button
              onClick={() => setActiveCategory(null)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-2xl text-sm font-bold transition-all ${activeCategory === null
                ? 'bg-white/10 text-white'
                : 'text-gray-500 hover:text-gray-300 hover:bg-white/5'
                }`}
            >
              All Categories
            </button>
            {CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                onClick={() => setActiveCategory(cat.id)}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-2xl text-sm font-bold transition-all ${activeCategory === cat.id
                  ? 'bg-white/10 text-white'
                  : 'text-gray-500 hover:text-gray-300 hover:bg-white/5'
                  }`}
              >
                <span className="shrink-0">{cat.icon}</span>
                <span className="truncate">{cat.name}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col gap-6 overflow-hidden min-h-0">
        <div className="flex flex-col md:flex-row gap-4 shrink-0">
          <form onSubmit={handleSearch} className="relative flex-1">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" size={20} />
            <input
              type="text"
              placeholder="Search mods..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="w-full pl-12 pr-4 py-3 bg-black/20 border border-white/5 rounded-2xl focus:outline-none focus:border-primary/50 transition-colors"
            />
          </form>

          <Select
            value={sortOrder}
            onChange={(val) => setSortOrder(val as SortOrder)}
            options={SORT_OPTIONS}
            className="w-56"
          />
        </div>

        <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-20">
            {loading ? (
              Array.from({ length: 12 }).map((_, i) => (
                <div key={i} className="bg-white/5 border border-white/5 rounded-[2rem] p-6 h-56 animate-pulse" />
              ))
            ) : results.length > 0 ? (
              results.map((project) => {
                const isSelected = selectedMods.has(project.id)
                return (
                  <motion.div
                    key={`${project.provider}-${project.id}`}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    className={`relative bg-surface border transition-all group flex flex-col p-6 rounded-[2rem] ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
                      }`}
                  >
                    {isSelected && (
                      <div className="absolute -top-2 -right-2 bg-primary text-white p-1.5 rounded-full shadow-lg z-10">
                        <Check size={16} strokeWidth={3} />
                      </div>
                    )}

                    <div className="flex items-start gap-4 mb-4">
                      {project.icon_url ? (
                        <img src={project.icon_url} alt="" className="w-16 h-16 rounded-2xl object-cover bg-black/20 shadow-xl" />
                      ) : (
                        <div className="w-16 h-16 rounded-2xl bg-primary/10 text-primary flex items-center justify-center shadow-xl">
                          <Package size={32} />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <h3 className="font-black text-white truncate text-lg">
                          {project.title}
                        </h3>
                        <div className="flex items-center gap-2 text-xs text-gray-500 mt-1 font-medium min-w-0">
                          <span className="flex items-center gap-1 min-w-0 flex-1">
                            <User size={12} className="text-primary shrink-0" />
                            <span className="truncate">{project.author || 'Unknown'}</span>
                          </span>
                          <span className="flex items-center gap-1 shrink-0">
                            <Download size={12} className="text-primary" />
                            {project.downloads.toLocaleString()}
                          </span>
                        </div>
                      </div>
                    </div>

                    <p className="text-sm text-gray-400 truncate mb-6 font-medium leading-relaxed">
                      {project.description}
                    </p>

                    <div className="flex items-center gap-2 mt-auto">
                      <button
                        onClick={() => setSelectedProject(project)}
                        className="flex-1 py-3 bg-white/5 hover:bg-white/10 text-gray-300 rounded-xl text-sm font-bold transition-all flex items-center justify-center gap-2"
                      >
                        Details
                        <ChevronRight size={16} />
                      </button>
                      <button
                        onClick={() => toggleModSelection(project)}
                        className={`px-4 py-3 rounded-xl text-sm font-black transition-all flex items-center gap-2 ${isSelected
                          ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
                          : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-[1.02]'
                          }`}
                      >
                        {isSelected ? 'Remove' : 'Select'}
                      </button>
                    </div>
                  </motion.div>
                )
              })
            ) : !loading ? (
              <div className="col-span-full py-20 flex flex-col items-center justify-center text-center">
                <div className="p-8 bg-white/5 rounded-[2.5rem] mb-6">
                  <Search size={64} className="text-gray-600" />
                </div>
                <h3 className="text-2xl font-black text-white mb-2 tracking-tight">No mods found</h3>
                <p className="text-gray-500 font-medium">Try a different search term or filters.</p>
              </div>
            ) : null}
          </div>

          {/* Pagination */}
          {results.length > 0 && (
            <div className="flex items-center justify-center gap-4 py-8">
              <button
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1 || loading}
                className="p-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:cursor-not-allowed text-gray-400 rounded-xl transition-all border border-white/5"
              >
                <ChevronLeft size={24} />
              </button>

              <div className="flex items-center gap-2">
                {[...Array(Math.min(5, page + 2))].map((_, i) => {
                  const pageNum = Math.max(1, page > 3 ? page - 2 + i : i + 1)
                  return (
                    <button
                      key={pageNum}
                      onClick={() => setPage(pageNum)}
                      className={`w-10 h-10 rounded-xl font-bold transition-all ${page === pageNum
                        ? 'bg-primary text-white shadow-lg shadow-primary/20'
                        : 'bg-white/5 text-gray-500 hover:text-white hover:bg-white/10'
                        }`}
                    >
                      {pageNum}
                    </button>
                  )
                })}
              </div>

              <button
                onClick={() => setPage(p => p + 1)}
                disabled={results.length < PAGE_SIZE || loading}
                className="p-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:cursor-not-allowed text-gray-400 rounded-xl transition-all border border-white/5"
              >
                <ChevronRight size={24} />
              </button>
            </div>
          )}
        </div>

        {/* Floating Bottom Bar for Review */}
        <AnimatePresence>
          {selectedMods.size > 0 && (
            <motion.div
              initial={{ y: 100, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              exit={{ y: 100, opacity: 0 }}
              className="absolute bottom-8 left-1/2 -translate-x-1/2 w-full max-w-lg px-6 z-50"
            >
              <div className="bg-surface/80 backdrop-blur-xl border border-white/10 p-4 rounded-3xl shadow-2xl flex items-center justify-between gap-6">
                <div className="flex items-center gap-4 pl-2">
                  <div className="w-12 h-12 rounded-2xl bg-primary/20 text-primary flex items-center justify-center font-black text-xl">
                    {selectedMods.size}
                  </div>
                  <div>
                    <div className="text-white font-bold">Mods selected</div>
                    <div className="text-xs text-gray-500 font-medium">Ready to review and install</div>
                  </div>
                </div>
                <button
                  onClick={handleReview}
                  disabled={isResolvingDeps}
                  className="px-8 py-3 bg-primary text-white rounded-2xl font-black shadow-lg shadow-primary/20 hover:scale-105 transition-all relative overflow-hidden disabled:opacity-80 disabled:hover:scale-100"
                >
                  <span className={isResolvingDeps ? 'opacity-0' : 'opacity-100'}>
                    Review and Confirm
                  </span>

                  {isResolvingDeps && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                      <span className="text-[10px] uppercase tracking-tighter mb-1">Finding Dependencies...</span>
                      <div className="w-24 h-1 bg-white/20 rounded-full overflow-hidden">
                        <motion.div
                          className="h-full bg-white"
                          initial={{ width: "0%" }}
                          animate={{ width: "100%" }}
                          transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                        />
                      </div>
                    </div>
                  )}
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <AnimatePresence>
        {selectedProject && (
          <ModDetailsModal
            project={selectedProject}
            instanceId={instanceId}
            onClose={() => setSelectedProject(null)}
            onInstall={() => {
              toggleModSelection(selectedProject)
              setSelectedProject(null)
            }}
            isSelected={selectedMods.has(selectedProject.id)}
          />
        )}
        {showReview && (
          <ModReviewModal
            selectedMods={Array.from(selectedMods.values())}
            preFetchedDependencies={resolvedDeps}
            instanceId={instanceId}
            onClose={() => setShowReview(false)}
            onConfirm={handleConfirmInstall}
            isInstalling={isInstalling}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
