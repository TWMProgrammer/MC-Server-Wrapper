import { Search, LayoutGrid, List } from 'lucide-react'
import { Select } from '../components/Select'
import { SortOrder } from '../types'
import { SORT_OPTIONS } from './marketplace_constants'

interface MarketplaceHeaderProps {
  query: string;
  setQuery: (query: string) => void;
  sortOrder: SortOrder;
  setSortOrder: (sort: SortOrder) => void;
  onSearch: (e?: React.FormEvent) => void;
  viewMode: 'grid' | 'list';
  setViewMode: (mode: 'grid' | 'list') => void;
}

export function MarketplaceHeader({
  query,
  setQuery,
  sortOrder,
  setSortOrder,
  onSearch,
  viewMode,
  setViewMode
}: MarketplaceHeaderProps) {
  return (
    <div className="flex flex-col md:flex-row gap-4 shrink-0">
      <form onSubmit={(e) => { e.preventDefault(); onSearch(e); }} className="relative flex-1">
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" size={20} />
        <input
          type="text"
          placeholder="Search mods..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full pl-12 pr-4 py-3 bg-black/20 border border-white/5 rounded-2xl focus:outline-none focus:border-primary/50 transition-colors"
        />
      </form>

      <div className="flex items-center gap-2">
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
  )
}
