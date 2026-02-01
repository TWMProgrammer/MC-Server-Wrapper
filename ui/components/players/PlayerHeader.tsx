import { motion, AnimatePresence } from 'framer-motion'
import { Search, UserPlus, X, Edit3, Plus, RefreshCw } from 'lucide-react'
import { cn } from '../../utils'
import { PlayerSubTab } from '../../PlayersTab'

interface PlayerHeaderProps {
  activeSubTab: PlayerSubTab;
  setActiveSubTab: (tab: PlayerSubTab) => void;
  subTabs: { id: PlayerSubTab; label: string; icon: any; color: string }[];
  isAddModalOpen: boolean;
  setIsAddModalOpen: (open: boolean) => void;
  addModalRef: React.RefObject<HTMLDivElement | null>;
  newUsername: string;
  setNewUsername: (name: string) => void;
  handleAddPlayer: (e: React.FormEvent) => void;
  handleRawEdit: () => void;
  adding: boolean;
}

export function PlayerHeader({
  activeSubTab,
  setActiveSubTab,
  subTabs,
  isAddModalOpen,
  setIsAddModalOpen,
  addModalRef,
  newUsername,
  setNewUsername,
  handleAddPlayer,
  handleRawEdit,
  adding
}: PlayerHeaderProps) {
  return (
    <div className="flex flex-col xl:flex-row items-start xl:items-center justify-between gap-6 pb-6 border-b border-black/5 dark:border-white/5">
      <div className="flex flex-wrap gap-2">
        {subTabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveSubTab(tab.id)}
            className={cn(
              "relative px-5 py-2.5 rounded-xl transition-all flex items-center gap-2.5 text-sm font-bold uppercase tracking-wider",
              activeSubTab === tab.id
                ? "bg-black/10 dark:bg-white/10 text-gray-900 dark:text-white shadow-lg"
                : "text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5"
            )}
          >
            <tab.icon size={18} className={cn(activeSubTab === tab.id ? tab.color : "text-current")} />
            {tab.label}
            {activeSubTab === tab.id && (
              <motion.div
                layoutId="active-player-tab"
                className="absolute inset-0 bg-black/5 dark:bg-white/5 rounded-xl -z-10"
                transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
              />
            )}
          </button>
        ))}
      </div>

      <div className="flex items-center gap-3 w-full xl:w-auto">
        <div className="relative flex-1 xl:w-64">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20" size={18} />
          <input
            type="text"
            placeholder="Search players..."
            className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-2xl py-3 pl-12 pr-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20"
          />
        </div>
        {activeSubTab !== 'all' && (
          <div className="flex items-center gap-3 relative" ref={addModalRef}>
            <motion.button
              whileHover={{ scale: 1.02, translateY: -2 }}
              whileTap={{ scale: 0.98 }}
              className={cn(
                "flex items-center gap-2 px-6 py-3 border rounded-2xl transition-all text-xs font-black uppercase tracking-widest",
                isAddModalOpen
                  ? "bg-primary text-white border-primary shadow-glow-primary"
                  : "bg-black/5 dark:bg-white/5 text-gray-900 dark:text-white border-black/10 dark:border-white/10 hover:bg-black/10 dark:hover:bg-white/10"
              )}
              onClick={() => setIsAddModalOpen(!isAddModalOpen)}
            >
              {isAddModalOpen ? <X size={18} /> : <UserPlus size={18} />}
              {activeSubTab === 'banned-ips' ? 'Ban IP' : 'Add Player'}
            </motion.button>

            <motion.button
              whileHover={{ scale: 1.02, translateY: -2 }}
              whileTap={{ scale: 0.98 }}
              className="flex items-center gap-2 px-6 py-3 bg-primary/10 text-primary border border-primary/20 rounded-2xl hover:bg-primary/20 transition-all text-xs font-black uppercase tracking-widest"
              onClick={handleRawEdit}
            >
              <Edit3 size={18} />
              Edit Raw List
            </motion.button>

            <AnimatePresence>
              {isAddModalOpen && (
                <motion.div
                  initial={{ opacity: 0, y: 10, scale: 0.95 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: 10, scale: 0.95 }}
                  className="absolute top-full mt-3 right-0 w-80 glass-panel p-5 rounded-2xl border border-black/10 dark:border-white/10 shadow-2xl z-50"
                >
                  <form onSubmit={handleAddPlayer} className="space-y-4">
                    <div className="flex flex-col gap-1.5">
                      <label className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/30 ml-1">
                        {activeSubTab === 'banned-ips' ? 'Ban IP Address' : `Add to ${activeSubTab}`}
                      </label>
                      <div className="relative group">
                        <UserPlus className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={18} />
                        <input
                          autoFocus
                          type="text"
                          placeholder={activeSubTab === 'banned-ips' ? "Enter IP address..." : "Enter Minecraft username..."}
                          className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl py-3 pl-11 pr-4 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20"
                          value={newUsername}
                          onChange={(e) => setNewUsername(e.target.value)}
                        />
                      </div>
                    </div>
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      type="submit"
                      disabled={adding || !newUsername.trim()}
                      className="w-full py-3 bg-primary hover:bg-primary-hover disabled:bg-black/5 dark:disabled:bg-white/5 disabled:text-gray-400 dark:disabled:text-white/20 disabled:cursor-not-allowed rounded-xl transition-all text-xs font-black uppercase tracking-widest text-white shadow-glow-primary flex items-center justify-center gap-2"
                    >
                      {adding ? (
                        <RefreshCw size={16} className="animate-spin" />
                      ) : (
                        <Plus size={16} />
                      )}
                      {adding ? 'Adding...' : 'Add Player'}
                    </motion.button>
                  </form>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )}
      </div>
    </div>
  )
}
