import { useMemo } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Trash2, User } from 'lucide-react'
import { PlayerEntry, OpEntry, BannedPlayerEntry, BannedIpEntry } from '../../types'
import { AppSettings } from '../../hooks/useAppSettings'

interface PlayerListTableProps {
  list: (PlayerEntry | OpEntry | BannedPlayerEntry | BannedIpEntry)[];
  onRemove: (identifier: string) => void;
  settings: AppSettings;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05
    }
  }
};

const itemVariants = {
  hidden: { opacity: 0, scale: 0.9, y: 10 },
  visible: {
    opacity: 1,
    scale: 1,
    y: 0,
    transition: {
      type: "spring" as const,
      stiffness: 300,
      damping: 25
    }
  },
  exit: {
    opacity: 0,
    scale: 0.9,
    transition: { duration: 0.2 }
  }
};

export function PlayerListTable({ list, onRemove, settings }: PlayerListTableProps) {
  return (
    <motion.div
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
    >
      <AnimatePresence mode="popLayout">
        {list.map((player) => {
          const identifier = 'name' in player ? player.name : player.ip;
          const uuid = 'uuid' in player ? player.uuid : undefined;

          const avatarUrl = useMemo(() => {
            if (!settings.download_player_heads) return null;

            // IPs don't have avatars in minotar generally, but we'll try username/uuid if available
            if (!('name' in player) && !('uuid' in player)) return null;

            const headIdentifier = settings.query_heads_by_username ? identifier : (uuid || identifier);
            const type = settings.use_helm_heads ? 'helm' : 'avatar';
            return `https://minotar.net/${type}/${headIdentifier}/48`;
          }, [identifier, uuid, settings.download_player_heads, settings.query_heads_by_username, settings.use_helm_heads, player]);

          return (
            <motion.div
              layout
              variants={itemVariants}
              key={identifier}
              className="glass-panel p-4 rounded-2xl flex items-center justify-between border border-black/5 dark:border-white/5 group hover:border-primary/30 transition-all hover:translate-y-[-2px]"
            >
              <div className="flex items-center gap-4">
                <div className="relative shrink-0">
                  {avatarUrl ? (
                    <img
                      src={avatarUrl}
                      alt={identifier}
                      className="w-12 h-12 rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10"
                    />
                  ) : (
                    <div className="w-12 h-12 rounded-xl bg-black/5 dark:bg-white/5 flex items-center justify-center text-gray-400 border border-black/5 dark:border-white/5">
                      <User size={24} />
                    </div>
                  )}
                </div>
                <div className="flex flex-col min-w-0">
                  <span className="font-bold text-gray-900 dark:text-white tracking-tight truncate">
                    {identifier}
                  </span>
                  {'level' in player && (
                    <span className="text-[10px] text-accent-amber font-black uppercase tracking-widest mt-0.5">
                      Level {(player as OpEntry).level}
                    </span>
                  )}
                  {'reason' in player && (
                    <span className="text-[10px] text-accent-rose font-black uppercase tracking-widest mt-0.5 truncate max-w-[120px]">
                      {player.reason}
                    </span>
                  )}
                </div>
              </div>
              <motion.button
                whileHover={{ scale: 1.1, translateY: -2 }}
                whileTap={{ scale: 0.9 }}
                onClick={() => onRemove(identifier)}
                className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-rose hover:bg-accent-rose/10 rounded-xl transition-all opacity-0 group-hover:opacity-100"
              >
                <Trash2 size={18} />
              </motion.button>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </motion.div>
  )
}
