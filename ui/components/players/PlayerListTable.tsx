import { motion } from 'framer-motion'
import { Trash2 } from 'lucide-react'
import { PlayerEntry, OpEntry, BannedPlayerEntry, BannedIpEntry } from '../../types'

interface PlayerListTableProps {
  list: (PlayerEntry | OpEntry | BannedPlayerEntry | BannedIpEntry)[];
  onRemove: (identifier: string) => void;
}

export function PlayerListTable({ list, onRemove }: PlayerListTableProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      {list.map((player, index) => (
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: index * 0.05 }}
          key={'name' in player ? player.name : player.ip}
          className="glass-panel p-4 rounded-2xl flex items-center justify-between border border-black/5 dark:border-white/5 group hover:border-primary/30 transition-all hover:translate-y-[-2px]"
        >
          <div className="flex items-center gap-4">
            <div className="relative shrink-0">
              <img
                src={`https://minotar.net/avatar/${'name' in player ? player.name : player.ip}/48`}
                alt={'name' in player ? player.name : player.ip}
                className="w-12 h-12 rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10"
              />
            </div>
            <div className="flex flex-col min-w-0">
              <span className="font-bold text-gray-900 dark:text-white tracking-tight truncate">
                {'name' in player ? player.name : player.ip}
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
            onClick={() => onRemove('name' in player ? player.name : player.ip)}
            className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-rose hover:bg-accent-rose/10 rounded-xl transition-all opacity-0 group-hover:opacity-100"
          >
            <Trash2 size={18} />
          </motion.button>
        </motion.div>
      ))}
    </div>
  )
}
