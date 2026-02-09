import { useState, useEffect, useRef } from 'react'
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
  LayoutGrid,
  List
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { Project, PluginProvider, SortOrder, SearchOptions, Instance, ResolvedDependency, PluginDependencies } from '../types'
import { useToast } from '../hooks/useToast'
import { formatNumber } from '../utils'
import { PluginDetailsModal } from './PluginDetailsModal'
import { ReviewModal } from './ReviewModal'
import { Select } from '../components/Select'
import { MarketplaceFloatingBar } from '../mods/MarketplaceFloatingBar'
import { InstallationProgressModal } from '../components/InstallationProgressModal'

interface MarketplaceProps {
  instanceId: string;
  onInstallSuccess?: () => void;
}

const MODRINTH_CATEGORIES = [
  { id: 'optimization', name: 'Optimization', icon: '⚡' },
  { id: 'utility', name: 'Utility', icon: '🛠️' },
  { id: 'worldgen', name: 'World Gen', icon: '🌍' },
  { id: 'management', name: 'Management', icon: '📋' },
  { id: 'economy', name: 'Economy', icon: '💰' },
  { id: 'chat', name: 'Chat', icon: '💬' },
  { id: 'game-mechanics', name: 'Mechanics', icon: '⚙️' },
  { id: 'library', name: 'Library', icon: '📚' },
  { id: 'magic', name: 'Magic', icon: '🪄' },
]

const SPIGET_CATEGORIES = [
  { id: '10', name: 'Admin', icon: '🛡️' },
  { id: '11', name: 'Chat', icon: '💬' },
  { id: '12', name: 'Economy', icon: '💰' },
  { id: '13', name: 'Gameplay', icon: '🎮' },
  { id: '14', name: 'Management', icon: '📋' },
  { id: '15', name: 'Protection', icon: '⚔️' },
  { id: '16', name: 'Utility', icon: '🛠️' },
  { id: '17', name: 'World Management', icon: '🌍' },
  { id: '18', name: 'Misc', icon: '📦' },
  { id: '19', name: 'Library', icon: '📚' },
]

const HANGAR_CATEGORIES = [
  { id: 'admin', name: 'Admin', icon: '🛡️' },
  { id: 'chat', name: 'Chat', icon: '💬' },
  { id: 'dev-tools', name: 'Dev Tools', icon: '🛠️' },
  { id: 'economy', name: 'Economy', icon: '💰' },
  { id: 'gameplay', name: 'Gameplay', icon: '🎮' },
  { id: 'games', name: 'Games', icon: '🕹️' },
  { id: 'protection', name: 'Protection', icon: '⚔️' },
  { id: 'roleplay', name: 'Roleplay', icon: '🎭' },
  { id: 'world-management', name: 'World', icon: '🌍' },
  { id: 'misc', name: 'Misc', icon: '📦' },
]

const SORT_OPTIONS = [
  { value: 'Relevance', label: 'Sort by Relevance' },
  { value: 'Downloads', label: 'Sort by Downloads' },
  { value: 'Follows', label: 'Sort by Follows' },
  { value: 'Newest', label: 'Sort by Newest' },
  { value: 'Updated', label: 'Sort by Updated' },
]

export function Marketplace({ instanceId, onInstallSuccess }: MarketplaceProps) {
  const [query, setQuery] = useState('')
  const [provider, setProvider] = useState<PluginProvider>('Modrinth')
  const [results, setResults] = useState<Project[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [selectedPlugins, setSelectedPlugins] = useState<Map<string, Project>>(new Map())
  const [showReview, setShowReview] = useState(false)
  const [isInstalling, setIsInstalling] = useState(false)
  const [isResolvingDeps, setIsResolvingDeps] = useState(false)
  const [resolvedDeps, setResolvedDeps] = useState<ResolvedDependency[]>([])
  const [activeCategory, setActiveCategory] = useState<string | null>(null)
  const [sortOrder, setSortOrder] = useState<SortOrder>('Relevance')
  const [page, setPage] = useState(1)
  const [instance, setInstance] = useState<Instance | null>(null)
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')
  const [pageSize, setPageSize] = useState(25)
  const [installProgress, setInstallProgress] = useState({ current: 0, total: 0, name: '' })

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

  const isVelocity = instance?.mod_loader?.toLowerCase() === 'velocity'

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
        offset: (page - 1) * pageSize,
        limit: pageSize,
        // Don't filter by version for Velocity as it's a proxy and plugins are version-independent usually,
        // or the version numbering is different from game versions.
        game_version: isVelocity ? undefined : instance?.version,
        loader: isVelocity ? 'velocity' : instance?.server_type?.toLowerCase(),
      }

      const searchResults = await invoke<Project[]>('search_plugins', {
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
    if (instance) {
      if (isVelocity && provider !== 'Modrinth') {
        setProvider('Modrinth')
      } else {
        handleSearch()
      }
    }
  }, [provider, activeCategory, sortOrder, page, instance, pageSize, isVelocity])

  // Reset page when provider, category or sort changes
  useEffect(() => {
    setPage(1)
  }, [provider, activeCategory, sortOrder, query, pageSize])

  const togglePluginSelection = (project: Project) => {
    const newSelection = new Map(selectedPlugins)
    if (newSelection.has(project.id)) {
      newSelection.delete(project.id)
    } else {
      newSelection.set(project.id, project)
    }
    setSelectedPlugins(newSelection)
  }

  const handleReview = async () => {
    setIsResolvingDeps(true)
    const allDeps: Map<string, ResolvedDependency> = new Map()
    const seenIds = new Set(Array.from(selectedPlugins.keys()))
    const queue = Array.from(selectedPlugins.values())

    try {
      while (queue.length > 0) {
        const plugin = queue.shift()!
        const deps = await invoke<PluginDependencies>('get_plugin_dependencies', {
          instanceId,
          projectId: plugin.id,
          provider: plugin.provider
        })

        // Handle mandatory dependencies
        for (const depProject of deps.mandatory) {
          if (!seenIds.has(depProject.id)) {
            allDeps.set(depProject.id, { project: depProject, dependency_type: 'required' })
            seenIds.add(depProject.id)
            queue.push(depProject)
          }
        }

        // Handle optional dependencies
        for (const depProject of deps.optional) {
          if (!seenIds.has(depProject.id)) {
            allDeps.set(depProject.id, { project: depProject, dependency_type: 'optional' })
            seenIds.add(depProject.id)
            // We don't necessarily need to recurse for optional dependencies unless the user selects them,
            // but for simplicity in the review screen we can just resolve them one level.
            // If we want full recursion: queue.push(depProject)
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

  const handleConfirmInstall = async (plugins: Project[]) => {
    setIsInstalling(true)
    setInstallProgress({ current: 0, total: plugins.length, name: plugins[0]?.title || '' })
    setShowReview(false)

    try {
      for (let i = 0; i < plugins.length; i++) {
        const plugin = plugins[i]
        setInstallProgress(prev => ({ ...prev, current: i, name: plugin.title }))

        await invoke('install_plugin', {
          instanceId,
          projectId: plugin.id,
          provider: plugin.provider,
          versionId: null // Latest
        })
      }

      setInstallProgress(prev => ({ ...prev, current: plugins.length }))
      // Small delay to show completion state
      await new Promise(resolve => setTimeout(resolve, 1000))

      showToast(`Successfully installed ${plugins.length} plugins!`, 'success')
      setSelectedPlugins(new Map())
      onInstallSuccess?.()
    } catch (err) {
      console.error('Installation failed:', err)
      showToast('Installation failed: ' + err, 'error')
    } finally {
      setIsInstalling(false)
    }
  }

  const categories = provider === 'Modrinth'
    ? MODRINTH_CATEGORIES
    : provider === 'Spiget'
      ? SPIGET_CATEGORIES
      : HANGAR_CATEGORIES

  return (
    <div className="flex flex-1 gap-8 overflow-hidden min-h-0">
      {/* Sidebar */}
      <div className="w-64 flex flex-col gap-6 shrink-0 overflow-y-auto custom-scrollbar pr-2">
        <div className="space-y-4">
          <div className="flex items-center gap-2 text-sm font-bold text-gray-400 uppercase tracking-widest px-2">
            <Layers size={16} />
            Providers
          </div>
          <div className="space-y-1">
            {(['Modrinth', 'Spiget', 'Hangar'] as const).map((p) => {
              const isDisabled = isVelocity && p !== 'Modrinth';
              if (isVelocity && p !== 'Modrinth') return null; // Hide other providers for Velocity

              return (
                <button
                  key={p}
                  disabled={isDisabled}
                  onClick={() => {
                    if (isDisabled) return;
                    setProvider(p)
                    setQuery('') // Reset search query when switching providers
                    setActiveCategory(null) // Reset category when switching providers
                    setPage(1) // Reset page when switching providers
                  }}
                  className={`w-full flex items-center gap-3 px-4 py-3 rounded-2xl text-sm font-bold transition-all ${provider === p
                    ? 'bg-primary text-white shadow-lg shadow-primary/20'
                    : isDisabled
                      ? 'opacity-50 cursor-not-allowed text-gray-600'
                      : 'text-gray-500 hover:text-gray-300 hover:bg-white/5'
                    }`}
                >
                  <div className={`w-2 h-2 rounded-full ${p === 'Modrinth' ? 'bg-green-500' : p === 'Spiget' ? 'bg-orange-500' : 'bg-blue-500'}`} />
                  {p}
                </button>
              );
            })}
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
            {categories.map((cat) => (
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
              placeholder={isVelocity ? "Search Modrinth plugins for Velocity..." : `Search ${provider} plugins...`}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="w-full pl-12 pr-4 py-3 bg-black/20 border border-white/5 rounded-2xl focus:outline-none focus:border-primary/50 transition-colors"
            />
          </form>

          <div className="flex items-center gap-2">
            <Select
              value={pageSize.toString()}
              onChange={(val) => setPageSize(parseInt(val))}
              options={[
                { value: '25', label: '25 per page' },
                { value: '50', label: '50 per page' },
                { value: '100', label: '100 per page' },
              ]}
              className="w-32"
            />

            <div className="flex bg-black/20 border border-white/5 rounded-2xl p-1">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 rounded-xl transition-all ${viewMode === 'grid'
                  ? 'bg-white/10 text-white shadow-lg'
                  : 'text-gray-500 hover:text-gray-300'
                  }`}
                title="Grid View"
              >
                <LayoutGrid size={20} />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 rounded-xl transition-all ${viewMode === 'list'
                  ? 'bg-white/10 text-white shadow-lg'
                  : 'text-gray-500 hover:text-gray-300'
                  }`}
                title="List View"
              >
                <List size={20} />
              </button>
            </div>

            <Select
              value={sortOrder}
              onChange={(val) => setSortOrder(val as SortOrder)}
              options={SORT_OPTIONS}
              className="w-56"
            />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
          <div className={viewMode === 'grid'
            ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-4"
            : "flex flex-col gap-3 pb-4"
          }>
            {loading ? (
              Array.from({ length: pageSize }).map((_, i) => (
                <div key={i} className={`bg-white/5 border border-white/5 animate-pulse ${viewMode === 'grid' ? 'rounded-[2rem] h-56' : 'rounded-2xl h-20'}`} />
              ))
            ) : results.length > 0 ? (
              results.map((project) => {
                const isSelected = selectedPlugins.has(project.id)
                if (viewMode === 'list') {
                  return (
                    <motion.div
                      key={`${project.provider}-${project.id}`}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      className={`relative bg-surface border transition-all group flex items-center p-4 rounded-2xl gap-4 ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
                        }`}
                    >
                      <div className="relative shrink-0">
                        {project.icon_url ? (
                          <img src={project.icon_url} alt="" className="w-12 h-12 rounded-xl object-cover bg-black/20 shadow-lg" />
                        ) : (
                          <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center text-gray-500 shadow-lg">
                            <Package size={24} />
                          </div>
                        )}
                        {isSelected && (
                          <div className="absolute -top-2 -right-2 bg-primary text-white p-1 rounded-full shadow-lg z-10 border-2 border-[#0a0a0c]">
                            <Check size={8} strokeWidth={4} />
                          </div>
                        )}
                      </div>

                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-3">
                          <h3 className="font-bold text-white truncate text-base group-hover:text-primary transition-colors">
                            {project.title}
                          </h3>
                          <div className="flex items-center gap-1.5 px-2 py-0.5 bg-white/5 rounded-lg shrink-0">
                            <div className={`w-1.5 h-1.5 rounded-full ${project.provider === 'Modrinth' ? 'bg-green-500' : project.provider === 'Spiget' ? 'bg-orange-500' : 'bg-blue-500'}`} />
                            <span className="text-[9px] font-black uppercase tracking-widest text-gray-500">{project.provider}</span>
                          </div>
                        </div>
                        <p className="text-sm text-gray-400 line-clamp-1 font-medium mt-0.5">
                          {project.description}
                        </p>
                      </div>

                      <div className="flex items-center gap-6 shrink-0 ml-4 px-6 border-l border-white/5">
                        <div className="flex flex-col items-center gap-1">
                          <Download size={14} className="text-primary" />
                          <span className="text-[10px] font-bold text-gray-500">{formatNumber(project.downloads)}</span>
                        </div>
                        <div className="flex flex-col items-center gap-1">
                          <Star size={14} className="text-primary" />
                          <span className="text-[10px] font-bold text-gray-500">{(project.downloads / 5000).toFixed(0)}</span>
                        </div>
                      </div>

                      <div className="flex items-center gap-2 shrink-0 ml-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            setSelectedProject(project)
                          }}
                          className="p-2.5 bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white rounded-xl transition-all"
                        >
                          <ExternalLink size={18} />
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            togglePluginSelection(project)
                          }}
                          className={`px-5 py-2.5 rounded-xl text-xs font-black transition-all ${isSelected
                            ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
                            : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-105'
                            }`}
                        >
                          {isSelected ? 'Remove' : 'Select'}
                        </button>
                      </div>
                    </motion.div>
                  )
                }

                return (
                  <motion.div
                    key={`${project.provider}-${project.id}`}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    className={`relative bg-surface border transition-all group flex flex-col p-6 rounded-[2rem] h-56 ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
                      }`}
                  >
                    {isSelected && (
                      <div className="absolute top-3 right-3 bg-primary text-white p-1.5 rounded-full shadow-lg z-10 border-2 border-[#0a0a0c]">
                        <Check size={12} strokeWidth={4} />
                      </div>
                    )}

                    <div className="flex items-start gap-4 mb-4">
                      {project.icon_url ? (
                        <img src={project.icon_url} alt="" className="w-16 h-16 rounded-2xl object-cover bg-black/20 shadow-xl" />
                      ) : (
                        <div className="w-16 h-16 rounded-2xl bg-white/5 flex items-center justify-center text-gray-500 shadow-xl">
                          <Package size={32} />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <h3 className="font-bold text-white truncate text-base group-hover:text-primary transition-colors">{project.title}</h3>
                        <div className="flex items-center gap-2 mt-1">
                          <div className={`w-2 h-2 rounded-full ${project.provider === 'Modrinth' ? 'bg-green-500' : project.provider === 'Spiget' ? 'bg-orange-500' : 'bg-blue-500'}`} />
                          <span className="text-[10px] font-black uppercase tracking-widest text-gray-500">{project.provider}</span>
                        </div>
                      </div>
                    </div>

                    <p className="text-sm text-gray-400 line-clamp-2 mb-4 font-medium leading-relaxed flex-1">
                      {project.description}
                    </p>

                    <div className="flex items-center justify-between pt-4 border-t border-white/5 gap-2">
                      <div className="flex items-center gap-3 min-w-0 overflow-hidden">
                        <div className="flex items-center gap-1 text-gray-500 shrink-0">
                          <Download size={14} className="text-primary" />
                          <span className="text-[10px] font-bold truncate">{formatNumber(project.downloads)}</span>
                        </div>
                        <div className="flex items-center gap-1 text-gray-500 shrink-0">
                          <Star size={14} className="text-primary" />
                          <span className="text-[10px] font-bold truncate">{(project.downloads / 5000).toFixed(0)}</span>
                        </div>
                      </div>

                      <div className="flex items-center gap-2 shrink-0">
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            setSelectedProject(project)
                          }}
                          className="p-2 bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white rounded-xl transition-all"
                        >
                          <ExternalLink size={16} />
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            togglePluginSelection(project)
                          }}
                          className={`px-4 py-2 rounded-xl text-xs font-black transition-all ${isSelected
                            ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
                            : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-105'
                            }`}
                        >
                          {isSelected ? 'Remove' : 'Select'}
                        </button>
                      </div>
                    </div>
                  </motion.div>
                )
              })
            ) : !loading ? (
              <div className="col-span-full py-20 flex flex-col items-center justify-center text-center">
                <div className="p-8 bg-white/5 rounded-[2.5rem] mb-6">
                  <Search size={64} className="text-gray-600" />
                </div>
                <h3 className="text-2xl font-black text-white mb-2 tracking-tight">No results found</h3>
                <p className="text-gray-500 font-medium">Try a different search term or category.</p>
              </div>
            ) : null}
          </div>
        </div>

        {/* Pagination & Floating Bar */}
        <div className="shrink-0 relative flex items-center justify-center min-h-[80px] py-4 border-t border-white/5 bg-white/[0.02] rounded-b-[2rem]">
          {results.length > 0 && (
            <div className="flex items-center justify-center gap-4">
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
                disabled={results.length < pageSize || loading}
                className="p-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:cursor-not-allowed text-gray-400 rounded-xl transition-all border border-white/5"
              >
                <ChevronRight size={24} />
              </button>
            </div>
          )}

          <div className="absolute right-6 top-1/2 -translate-y-1/2">
            <MarketplaceFloatingBar
              selectedCount={selectedPlugins.size}
              isResolvingDeps={isResolvingDeps}
              onReview={handleReview}
              label="Plugins"
            />
          </div>
        </div>
      </div>

      <AnimatePresence>
        {selectedProject && (
          <PluginDetailsModal
            project={selectedProject}
            instanceId={instanceId}
            onClose={() => setSelectedProject(null)}
            onInstall={() => {
              togglePluginSelection(selectedProject)
              setSelectedProject(null)
            }}
            isSelected={selectedPlugins.has(selectedProject.id)}
          />
        )}
        {showReview && (
          <ReviewModal
            selectedPlugins={Array.from(selectedPlugins.values())}
            preFetchedDependencies={resolvedDeps}
            instanceId={instanceId}
            onClose={() => setShowReview(false)}
            onConfirm={handleConfirmInstall}
            isInstalling={isInstalling}
          />
        )}
        <InstallationProgressModal
          isOpen={isInstalling}
          currentCount={installProgress.current}
          totalCount={installProgress.total}
          currentName={installProgress.name}
          type="plugin"
        />
      </AnimatePresence>
    </div>
  )
}
