import { useMemo } from 'react'
import { motion } from 'framer-motion'
import { FileText, Shield, Ban, User } from 'lucide-react'
import { cn } from '../../utils'
import { AppSettings } from '../../hooks/useAppSettings'

interface PlayerCardProps {
  player: {
    name: string;
    uuid?: string;
    isOnline: boolean;
    isWhitelisted: boolean;
    isOp: boolean;
    isBanned: boolean;
  };
  index: number;
  onQuickAdd: (username: string, listType: 'whitelist' | 'ops' | 'banned-players') => void;
  settings: AppSettings;
}

export function PlayerCard({ player, index, onQuickAdd, settings }: PlayerCardProps) {
  const avatarUrl = useMemo(() => {
    if (!settings.download_player_heads) return null;

    const identifier = settings.query_heads_by_username ? player.name : (player.uuid || player.name);
    const type = settings.use_helm_heads ? 'helm' : 'avatar';
    return `https://minotar.net/${type}/${identifier}/48`;
  }, [player.name, player.uuid, settings.download_player_heads, settings.query_heads_by_username, settings.use_helm_heads]);

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay: index * 0.05 }}
      key={player.name}
      className="glass-panel p-4 rounded-2xl flex items-center justify-between border border-black/5 dark:border-white/5 group hover:border-primary/30 transition-all hover:translate-y-[-2px]"
    >
      <div className="flex items-center gap-4">
        <div className="relative shrink-0">
          {avatarUrl ? (
            <img
              src={avatarUrl}
              alt={player.name}
              className="w-12 h-12 rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10"
            />
          ) : (
            <div className="w-12 h-12 rounded-xl bg-black/5 dark:bg-white/5 flex items-center justify-center text-gray-400 border border-black/5 dark:border-white/5">
              <User size={24} />
            </div>
          )}
          <span className={cn(
            "absolute -bottom-1 -right-1 w-4 h-4 rounded-full border-2 shadow-sm transition-all duration-500",
            player.isOnline
              ? "bg-emerald-500 border-white dark:border-[#0a0a0a] shadow-glow-emerald"
              : "bg-gray-400 border-white dark:border-[#0a0a0a]"
          )}></span>
        </div>
        <div className="flex flex-col min-w-0">
          <span className="font-bold text-gray-900 dark:text-white tracking-tight truncate">{player.name}</span>
          <div className="flex items-center gap-1.5 mt-0.5">
            {player.isWhitelisted && (
              <div title="Whitelisted" className="text-primary">
                <FileText size={12} />
              </div>
            )}
            {player.isOp && (
              <div title="Operator" className="text-accent-amber">
                <Shield size={12} />
              </div>
            )}
            {player.isBanned && (
              <div title="Banned" className="text-accent-rose">
                <Ban size={12} />
              </div>
            )}
            {!player.isWhitelisted && !player.isOp && !player.isBanned && (
              <span className="text-[10px] text-gray-400 dark:text-white/20 font-black uppercase tracking-widest">
                Neutral
              </span>
            )}
          </div>
        </div>
      </div>
      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all translate-x-2 group-hover:translate-x-0">
        {!player.isWhitelisted && (
          <motion.button
            whileHover={{ scale: 1.1, translateY: -2 }}
            whileTap={{ scale: 0.9 }}
            onClick={() => onQuickAdd(player.name, 'whitelist')}
            title="Add to Whitelist"
            className="p-2 text-gray-400 dark:text-white/40 hover:text-primary hover:bg-primary/10 rounded-xl transition-all"
          >
            <FileText size={18} />
          </motion.button>
        )}
        {!player.isOp && (
          <motion.button
            whileHover={{ scale: 1.1, translateY: -2 }}
            whileTap={{ scale: 0.9 }}
            onClick={() => onQuickAdd(player.name, 'ops')}
            title="Make Operator"
            className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-amber hover:bg-accent-amber/10 rounded-xl transition-all"
          >
            <Shield size={18} />
          </motion.button>
        )}
        {!player.isBanned && (
          <motion.button
            whileHover={{ scale: 1.1, translateY: -2 }}
            whileTap={{ scale: 0.9 }}
            onClick={() => onQuickAdd(player.name, 'banned-players')}
            title="Ban Player"
            className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-rose hover:bg-accent-rose/10 rounded-xl transition-all"
          >
            <Ban size={18} />
          </motion.button>
        )}
      </div>
    </motion.div>
  )
}
