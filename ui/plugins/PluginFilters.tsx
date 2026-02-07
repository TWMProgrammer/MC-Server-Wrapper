import { Search, List, LayoutGrid, RefreshCw, Database } from 'lucide-react'

type ViewMode = 'table' | 'grid'

interface PluginFiltersProps {
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  viewMode: ViewMode;
  setViewMode: (mode: ViewMode) => void;
  onCheckUpdates: () => void;
  onRefresh: () => void;
  onOpenDatabaseExplorer: () => void;
  loading: boolean;
  checkingUpdates: boolean;
}

export function PluginFilters({
  searchQuery,
  setSearchQuery,
  viewMode,
  setViewMode,
  onCheckUpdates,
  onRefresh,
  onOpenDatabaseExplorer,
  loading,
  checkingUpdates
}: PluginFiltersProps) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div className="flex items-center gap-4 flex-1">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
          <input
            type="text"
            placeholder="Search installed plugins..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-black/20 border border-white/5 rounded-xl focus:outline-none focus:border-primary/50 transition-colors"
          />
        </div>
        <div className="flex items-center bg-white/5 p-1 rounded-xl border border-white/5">
          <button
            onClick={() => setViewMode('table')}
            className={`p-1.5 rounded-lg transition-all ${viewMode === 'table' ? 'bg-primary text-white shadow-lg shadow-primary/20' : 'text-gray-500 hover:text-gray-300'}`}
            title="Table View"
          >
            <List size={18} />
          </button>
          <button
            onClick={() => setViewMode('grid')}
            className={`p-1.5 rounded-lg transition-all ${viewMode === 'grid' ? 'bg-primary text-white shadow-lg shadow-primary/20' : 'text-gray-500 hover:text-gray-300'}`}
            title="Grid View"
          >
            <LayoutGrid size={18} />
          </button>
        </div>
        <button
          onClick={onCheckUpdates}
          disabled={checkingUpdates || loading}
          className="flex items-center gap-2 px-4 py-2 bg-primary/10 hover:bg-primary/20 text-primary rounded-xl transition-all border border-primary/20 disabled:opacity-50"
        >
          <RefreshCw size={18} className={checkingUpdates ? 'animate-spin' : ''} />
          <span className="font-medium">{checkingUpdates ? 'Checking...' : 'Check for Updates'}</span>
        </button>
        <button
          onClick={onRefresh}
          disabled={loading}
          className="p-2.5 bg-white/5 hover:bg-white/10 text-gray-400 rounded-xl transition-all border border-white/5"
          title="Refresh list"
        >
          <RefreshCw size={20} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>

      <button
        onClick={onOpenDatabaseExplorer}
        className="flex items-center gap-2 px-4 py-2 bg-white/5 hover:bg-white/10 text-gray-300 rounded-xl transition-all border border-white/5 font-bold group"
        title="Database Explorer"
      >
        <Database size={18} className="text-gray-500 group-hover:text-primary transition-colors" />
        <span>DB Explorer</span>
      </button>
    </div>
  )
}
