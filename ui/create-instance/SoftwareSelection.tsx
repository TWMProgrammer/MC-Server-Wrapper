import { motion } from 'framer-motion'
import { cn } from '../utils'
import { SERVER_TYPES } from './constants'

interface SoftwareSelectionProps {
  onSelect: (id: string) => void;
}

export function SoftwareSelection({ onSelect }: SoftwareSelectionProps) {
  const categories = ['Playable Server', 'Network Proxy', 'Other'] as const;

  return (
    <div className="flex-1 overflow-auto p-8 custom-scrollbar">
      <div className="space-y-10">
        {categories.map((category, catIdx) => (
          <div key={category} className="space-y-6">
            <h2 className="text-sm font-black text-gray-500 dark:text-white/40 uppercase tracking-[0.2em] px-2">{category}</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
              {SERVER_TYPES.filter(t => t.category === category).map((type, i) => (
                <motion.button
                  key={type.id}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{
                    duration: 0.4,
                    ease: [0.23, 1, 0.32, 1],
                    delay: (catIdx * 0.1) + (i * 0.05)
                  }}
                  whileHover={{
                    scale: 1.02,
                    translateY: -4,
                    transition: { duration: 0.2, ease: "easeOut" }
                  }}
                  whileTap={{
                    scale: 0.98,
                    transition: { duration: 0.1 }
                  }}
                  onClick={() => onSelect(type.id)}
                  className="flex flex-col gap-4 p-5 rounded-2xl bg-black/[0.02] dark:bg-white/[0.02] border border-black/5 dark:border-white/5 hover:bg-black/[0.04] dark:hover:bg-white/[0.05] hover:border-primary/30 transition-all duration-200 text-left group relative overflow-hidden"
                >
                  <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 blur-3xl rounded-full -mr-16 -mt-16 group-hover:bg-primary/10 transition-colors" />
                  <div className="p-4 rounded-2xl bg-black/10 dark:bg-black/40 w-fit group-hover:bg-primary/20 group-hover:text-primary transition-all shadow-inner-light">
                    {type.icon}
                  </div>
                  <div className="space-y-2 relative z-10">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-gray-900 dark:text-white group-hover:text-primary transition-colors">{type.name}</span>
                      {type.badge && (
                        <span className={cn("text-[9px] font-black uppercase tracking-wider px-2 py-0.5 rounded-full bg-black/5 dark:bg-white/5", type.badgeColor)}>
                          {type.badge}
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-gray-500 dark:text-white/40 leading-relaxed line-clamp-2 font-medium">
                      {type.description}
                    </p>
                  </div>
                </motion.button>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
