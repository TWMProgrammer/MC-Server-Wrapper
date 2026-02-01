import { motion } from 'framer-motion'
import { Database } from 'lucide-react'

export function EmptyState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center relative overflow-hidden">
      {/* Background decorative elements */}
      <div className="absolute top-1/4 -left-20 w-96 h-96 bg-primary/10 rounded-full blur-[100px] -z-10" />
      <div className="absolute bottom-1/4 -right-20 w-96 h-96 bg-accent-rose/5 rounded-full blur-[100px] -z-10" />

      <motion.div
        initial={{ opacity: 0, scale: 0.9, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
        className="flex flex-col items-center text-center px-6"
      >
        <div className="relative mb-10">
          <motion.div
            animate={{
              scale: [1, 1.1, 1],
              rotate: [0, 5, -5, 0]
            }}
            transition={{
              duration: 6,
              repeat: Infinity,
              ease: "easeInOut"
            }}
            className="p-10 rounded-[3rem] bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 shadow-2xl relative z-10 backdrop-blur-sm"
          >
            <Database size={80} className="text-primary" />
          </motion.div>
          {/* Glow effect */}
          <div className="absolute inset-0 bg-primary/20 blur-[50px] -z-10 rounded-full scale-75" />
        </div>

        <h2 className="text-4xl font-black text-gray-900 dark:text-white mb-4 tracking-tighter">
          Ready to <span className="text-transparent bg-clip-text bg-gradient-to-r from-primary to-accent-rose">Craft?</span>
        </h2>
        <p className="text-lg text-gray-500 dark:text-white/40 max-w-md leading-relaxed font-medium">
          Select a server instance from the sidebar to manage your world, or create a brand new one to start your adventure.
        </p>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5 }}
          className="mt-12 flex flex-col items-center gap-4"
        >
          <div className="flex items-center gap-3 px-6 py-3 rounded-2xl bg-black/5 dark:bg-white/[0.03] border border-black/5 dark:border-white/5 text-xs font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">
            <div className="w-1.5 h-1.5 rounded-full bg-primary shadow-glow-primary animate-pulse" />
            System Online & Ready
          </div>
        </motion.div>
      </motion.div>
    </div>
  )
}
