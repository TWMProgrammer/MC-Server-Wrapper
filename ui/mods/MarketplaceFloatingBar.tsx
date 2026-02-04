import { motion, AnimatePresence } from 'framer-motion'

interface MarketplaceFloatingBarProps {
  selectedCount: number;
  isResolvingDeps: boolean;
  onReview: () => void;
  label?: string;
}

export function MarketplaceFloatingBar({
  selectedCount,
  isResolvingDeps,
  onReview,
  label = "Mods"
}: MarketplaceFloatingBarProps) {
  return (
    <AnimatePresence>
      {selectedCount > 0 && (
        <motion.div
          initial={{ x: 50, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          exit={{ x: 50, opacity: 0 }}
          className="w-full max-w-md shrink-0"
        >
          <div className="bg-[#121214]/90 backdrop-blur-2xl border border-white/10 p-3 rounded-[2rem] shadow-xl flex items-center justify-between gap-4">
            <div className="flex items-center gap-3 pl-2">
              <div className="w-10 h-10 rounded-xl bg-primary/20 text-primary flex items-center justify-center font-black text-lg">
                {selectedCount}
              </div>
              <div className="min-w-0">
                <div className="text-white text-sm font-bold truncate">{label} selected</div>
                <div className="text-[10px] text-gray-500 font-medium truncate">Ready to install</div>
              </div>
            </div>
            <button
              onClick={onReview}
              disabled={isResolvingDeps}
              className="px-6 py-2.5 bg-primary text-white rounded-xl font-bold text-sm shadow-lg shadow-primary/20 hover:scale-105 transition-all relative overflow-hidden disabled:opacity-80 disabled:hover:scale-100 shrink-0"
            >
              <span className={isResolvingDeps ? 'opacity-0' : 'opacity-100'}>
                Review & Confirm
              </span>

              {isResolvingDeps && (
                <div className="absolute inset-0 flex flex-col items-center justify-center">
                  <div className="w-16 h-1 bg-white/20 rounded-full overflow-hidden">
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
