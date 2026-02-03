import { ChevronLeft, ChevronRight } from 'lucide-react'

interface MarketplacePaginationProps {
  page: number;
  setPage: (page: number | ((p: number) => number)) => void;
  hasMore: boolean;
  loading: boolean;
}

export function MarketplacePagination({
  page,
  setPage,
  hasMore,
  loading
}: MarketplacePaginationProps) {
  return (
    <div className="flex items-center justify-center gap-4 py-8">
      <button
        onClick={() => setPage(p => Math.max(1, p - 1))}
        disabled={page === 1 || loading}
        data-testid="prev-page-btn"
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
        disabled={!hasMore || loading}
        data-testid="next-page-btn"
        className="p-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:cursor-not-allowed text-gray-400 rounded-xl transition-all border border-white/5"
      >
        <ChevronRight size={24} />
      </button>
    </div>
  )
}
