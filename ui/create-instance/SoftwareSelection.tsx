import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { LayoutGrid, List, ChevronRight } from 'lucide-react'
import { cn } from '../utils'
import { SERVER_TYPES } from './constants'

interface SoftwareSelectionProps {
  onSelect: (id: string) => void;
}

export function SoftwareSelection({ onSelect }: SoftwareSelectionProps) {
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('list');
  const categories = ['Playable Server', 'Network Proxy', 'Other'] as const;

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="px-6 py-4 flex items-center justify-between border-b border-black/5 dark:border-white/5 bg-black/[0.01] dark:bg-white/[0.01]">
        <h2 className="text-[11px] font-black text-gray-400 dark:text-white/20 uppercase tracking-[0.2em]">Select Software</h2>
        <div className="flex items-center gap-1 p-1 rounded-lg bg-black/5 dark:bg-white/5 border border-black/5 dark:border-white/5">
          <button
            onClick={() => setViewMode('list')}
            className={cn(
              "p-1.5 rounded-md transition-all duration-200",
              viewMode === 'list'
                ? "bg-white dark:bg-white/10 text-primary shadow-sm"
                : "text-gray-400 hover:text-gray-600 dark:hover:text-white/60"
            )}
            title="List View"
          >
            <List size={14} />
          </button>
          <button
            onClick={() => setViewMode('grid')}
            className={cn(
              "p-1.5 rounded-md transition-all duration-200",
              viewMode === 'grid'
                ? "bg-white dark:bg-white/10 text-primary shadow-sm"
                : "text-gray-400 hover:text-gray-600 dark:hover:text-white/60"
            )}
            title="Grid View"
          >
            <LayoutGrid size={14} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6 custom-scrollbar">
        <div className="space-y-8">
          {categories.map((category, catIdx) => (
            <div key={category} className="space-y-4">
              <h2 className="text-[10px] font-black text-gray-500 dark:text-white/30 uppercase tracking-[0.2em] px-2">{category}</h2>

              <div className={cn(
                "gap-4",
                viewMode === 'grid'
                  ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3"
                  : "flex flex-col"
              )}>
                <AnimatePresence mode="popLayout">
                  {SERVER_TYPES.filter(t => t.category === category).map((type, i) => (
                    <motion.button
                      key={type.id}
                      layout
                      initial={{ opacity: 0, y: 20 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, scale: 0.95 }}
                      transition={{
                        duration: 0.4,
                        ease: [0.23, 1, 0.32, 1],
                        delay: (catIdx * 0.1) + (i * 0.05)
                      }}
                      whileHover={{
                        scale: viewMode === 'grid' ? 1.02 : 1.01,
                        translateY: viewMode === 'grid' ? -4 : -2,
                        transition: { duration: 0.2, ease: "easeOut" }
                      }}
                      whileTap={{
                        scale: 0.98,
                        transition: { duration: 0.1 }
                      }}
                      onClick={() => onSelect(type.id)}
                      className={cn(
                        "group relative overflow-hidden transition-all duration-200 text-left border border-black/5 dark:border-white/5 hover:border-primary/30",
                        viewMode === 'grid'
                          ? "flex flex-col gap-3 p-4 rounded-2xl bg-black/[0.02] dark:bg-white/[0.02] hover:bg-black/[0.04] dark:hover:bg-white/[0.05]"
                          : "flex flex-row items-center gap-4 p-3 rounded-xl bg-black/[0.01] dark:bg-white/[0.01] hover:bg-black/[0.03] dark:hover:bg-white/[0.04]"
                      )}
                    >
                      <div className={cn(
                        "absolute top-0 right-0 bg-primary/5 blur-3xl rounded-full -mr-12 -mt-12 group-hover:bg-primary/10 transition-colors",
                        viewMode === 'grid' ? "w-24 h-24" : "w-16 h-16"
                      )} />

                      <div className={cn(
                        "rounded-xl bg-black/10 dark:bg-black/40 w-fit group-hover:bg-primary/20 group-hover:text-primary transition-all shadow-inner-light flex items-center justify-center shrink-0 overflow-hidden",
                        viewMode === 'grid' ? "w-12 h-12" : "w-10 h-10"
                      )}>
                        {type.imageUrl ? (
                          <img
                            src={type.imageUrl}
                            alt={type.name}
                            className={cn(
                              "w-full h-full object-contain",
                              type.id === 'vanilla' || type.id === 'bedrock' ? "p-1.5" : "p-2"
                            )}
                          />
                        ) : (
                          type.icon
                        )}
                      </div>

                      <div className="flex-1 space-y-1 relative z-10 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-bold text-gray-900 dark:text-white group-hover:text-primary transition-colors truncate">
                            {type.name}
                          </span>
                          {type.badge && (
                            <span className={cn("text-[8px] font-black uppercase tracking-wider px-1.5 py-0.5 rounded-full bg-black/5 dark:bg-white/5 shrink-0", type.badgeColor)}>
                              {type.badge}
                            </span>
                          )}
                        </div>
                        <p className={cn(
                          "text-[11px] text-gray-500 dark:text-white/40 leading-relaxed font-medium",
                          viewMode === 'grid' ? "line-clamp-2" : "line-clamp-1"
                        )}>
                          {type.description}
                        </p>
                      </div>

                      {viewMode === 'list' && (
                        <div className="opacity-0 group-hover:opacity-100 transition-all duration-200 transform translate-x-2 group-hover:translate-x-0 pr-2 shrink-0">
                          <ChevronRight size={16} className="text-primary" />
                        </div>
                      )}
                    </motion.button>
                  ))}
                </AnimatePresence>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
