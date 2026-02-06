import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Project, ModProvider, SortOrder, SearchOptions, Instance } from '../types'
import { useToast } from '../hooks/useToast'

const DEFAULT_PAGE_SIZE = 16

export function useModSearch(instanceId: string, initialPageSize = 25) {
  const [query, setQuery] = useState('')
  const [provider, setProvider] = useState<ModProvider>('Modrinth')
  const [results, setResults] = useState<Project[]>([])
  const [loading, setLoading] = useState(false)
  const [activeCategory, setActiveCategory] = useState<string | null>(null)
  const [sortOrder, setSortOrder] = useState<SortOrder>('Downloads')
  const [page, setPage] = useState(1)
  const [instance, setInstance] = useState<Instance | null>(null)
  const [pageSize, setPageSize] = useState(initialPageSize)
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
        offset: (page - 1) * pageSize,
        limit: pageSize,
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
    if (instance) {
      handleSearch()
    }
  }, [provider, activeCategory, sortOrder, page, instance, pageSize])

  // Reset page when filters change
  useEffect(() => {
    setPage(1)
  }, [provider, activeCategory, sortOrder, query, pageSize])

  return {
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
    instance,
    pageSize,
    setPageSize,
    handleSearch,
  }
}
