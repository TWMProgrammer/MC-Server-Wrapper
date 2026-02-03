import { motion, AnimatePresence } from 'framer-motion'

interface MarketplaceFloatingBarProps {
  selectedCount: number;
  isResolvingDeps: boolean;
  onReview: () => void;
}

export function MarketplaceFloatingBar({
  selectedCount,
  isResolvingDeps,
  onReview
}: MarketplaceFloatingBarProps) {
  return (
    <AnimatePresence>
      {selectedCount > 0 && (
        <motion.div
          initial={{ y: 100, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          exit={{ y: 100, opacity: 0 }}
          className="absolute bottom-8 left-1/2 -translate-x-1/2 w-full max-w-lg px-6 z-50"
        >
          <div className="bg-surface/80 backdrop-blur-xl border border-white/10 p-4 rounded-3xl shadow-2xl flex items-center justify-between gap-6">
            <div className="flex items-center gap-4 pl-2">
              <div className="w-12 h-12 rounded-2xl bg-primary/20 text-primary flex items-center justify-center font-black text-xl">
                {selectedCount}
              </div>
              <div>
                <div className="text-white font-bold">Mods selected</div>
                <div className="text-xs text-gray-500 font-medium">Ready to review and install</div>
              </div>
            </div>
            <button
              onClick={onReview}
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
  )
}
