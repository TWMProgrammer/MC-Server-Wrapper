import { useState, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Search } from 'lucide-react'
import { AnimatePresence } from 'framer-motion'
import { Project, ResolvedDependency } from '../types'
import { useToast } from '../hooks/useToast'
import { ModDetailsModal } from './ModDetailsModal'
import { ModReviewModal } from './ModReviewModal'
import { useModSearch } from './useModSearch'
import { MarketplaceSidebar } from './MarketplaceSidebar'
import { MarketplaceHeader } from './MarketplaceHeader'
import { ModCard } from './ModCard'
import { MarketplacePagination } from './MarketplacePagination'
import { MarketplaceFloatingBar } from './MarketplaceFloatingBar'
import { useGridPageSize } from '../hooks/useGridPageSize'

interface ModMarketplaceProps {
  instanceId: string;
  onInstallSuccess?: () => void;
}

export function ModMarketplace({ instanceId, onInstallSuccess }: ModMarketplaceProps) {
  const gridContainerRef = useRef<HTMLDivElement>(null)
  const pageSize = useGridPageSize(gridContainerRef)

  const {
    query,
    setQuery,
    provider,
    setProvider,
    results,
    loading,
    activeCategory,
    setActiveCategory,
    sortOrder,
    setSortOrder,
    page,
    setPage,
    handleSearch
  } = useModSearch(instanceId, pageSize)

  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [selectedMods, setSelectedMods] = useState<Map<string, Project>>(new Map())
  const [showReview, setShowReview] = useState(false)
  const [isInstalling, setIsInstalling] = useState(false)
  const [isResolvingDeps, setIsResolvingDeps] = useState(false)
  const [resolvedDeps, setResolvedDeps] = useState<ResolvedDependency[]>([])

  const { showToast } = useToast()

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
    const allDeps: Map<string, ResolvedDependency> = new Map()
    const seenIds = new Set(Array.from(selectedMods.keys()))
    const queue = Array.from(selectedMods.values())

    try {
      while (queue.length > 0) {
        const mod = queue.shift()!

        const deps = await invoke<ResolvedDependency[]>('get_mod_dependencies', {
          instanceId,
          projectId: mod.id,
          provider: mod.provider
        })

        for (const dep of deps) {
          if (!seenIds.has(dep.project.id)) {
            allDeps.set(dep.project.id, dep)
            seenIds.add(dep.project.id)
            queue.push(dep.project)
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
      <MarketplaceSidebar
        provider={provider}
        setProvider={setProvider}
        activeCategory={activeCategory}
        setActiveCategory={setActiveCategory}
      />

      <div className="flex-1 flex flex-col gap-6 overflow-hidden min-h-0">
        <MarketplaceHeader
          query={query}
          setQuery={setQuery}
          sortOrder={sortOrder}
          setSortOrder={setSortOrder}
          onSearch={handleSearch}
        />

        <div ref={gridContainerRef} className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-4">
            {loading ? (
              Array.from({ length: pageSize }).map((_, i) => (
                <div key={i} className="bg-white/5 border border-white/5 rounded-[2rem] p-6 h-56 animate-pulse" />
              ))
            ) : results.length > 0 ? (
              results.map((project) => (
                <ModCard
                  key={`${project.provider}-${project.id}`}
                  project={project}
                  isSelected={selectedMods.has(project.id)}
                  onSelect={toggleModSelection}
                  onShowDetails={setSelectedProject}
                />
              ))
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
        </div>

        <div className="shrink-0 relative flex items-center justify-center min-h-[80px] py-4 border-t border-white/5 bg-white/[0.02] rounded-b-[2rem]">
          {results.length > 0 && (
            <MarketplacePagination
              page={page}
              setPage={setPage}
              hasMore={results.length >= pageSize}
              loading={loading}
            />
          )}

          <div className="absolute right-6 top-1/2 -translate-y-1/2">
            <MarketplaceFloatingBar
              selectedCount={selectedMods.size}
              isResolvingDeps={isResolvingDeps}
              onReview={handleReview}
            />
          </div>
        </div>
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
