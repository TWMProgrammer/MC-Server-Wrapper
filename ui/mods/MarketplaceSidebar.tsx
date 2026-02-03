import { Globe, Tag } from 'lucide-react'
import { ModProvider } from '../types'
import { CATEGORIES } from './marketplace_constants'

interface MarketplaceSidebarProps {
  provider: ModProvider;
  setProvider: (provider: ModProvider) => void;
  activeCategory: string | null;
  setActiveCategory: (category: string | null) => void;
}

export function MarketplaceSidebar({
  provider,
  setProvider,
  activeCategory,
  setActiveCategory
}: MarketplaceSidebarProps) {
  return (
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
              data-testid={`category-${cat.id}`}
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
  )
}
