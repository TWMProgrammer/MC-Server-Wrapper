import { useState, useEffect } from 'react'
import { Trash2, UserPlus, FileText, Shield, Ban, Globe, Activity, Plus, Search, Info, RefreshCw } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { AllPlayerLists } from './types'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'

interface PlayersTabProps {
  instanceId: string;
}

type PlayerSubTab = 'live' | 'whitelist' | 'ops' | 'banned-players' | 'banned-ips';

export function PlayersTab({ instanceId }: PlayersTabProps) {
  const [activeSubTab, setActiveSubTab] = useState<PlayerSubTab>('live');
  const [lists, setLists] = useState<AllPlayerLists | null>(null);
  const [onlinePlayers, setOnlinePlayers] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newUsername, setNewUsername] = useState('');
  const [adding, setAdding] = useState(false);

  const fetchLists = async () => {
    try {
      const data: AllPlayerLists = await invoke('get_players', { instanceId });
      setLists(data);
      setError(null);
    } catch (err) {
      setError(err as string);
    }
  };

  const fetchOnlinePlayers = async () => {
    try {
      const data: string[] = await invoke('get_online_players', { instanceId });
      setOnlinePlayers(data);
    } catch (err) {
      console.error('Error fetching online players:', err);
    }
  };

  useEffect(() => {
    const init = async () => {
      setLoading(true);
      await Promise.all([fetchLists(), fetchOnlinePlayers()]);
      setLoading(false);
    };
    init();

    const interval = setInterval(() => {
      fetchLists();
      fetchOnlinePlayers();
    }, 5000);

    return () => clearInterval(interval);
  }, [instanceId]);

  const handleAddPlayer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newUsername.trim() || adding) return;

    try {
      setAdding(true);
      if (activeSubTab === 'banned-ips') {
        await invoke('add_banned_ip', { instanceId, ip: newUsername.trim() });
      } else {
        await invoke('add_player', { instanceId, listType: activeSubTab, username: newUsername.trim() });
      }
      setNewUsername('');
      await fetchLists();
    } catch (err) {
      alert(`Error adding player: ${err}`);
    } finally {
      setAdding(false);
    }
  };

  const handleRemovePlayer = async (identifier: string) => {
    try {
      await invoke('remove_player', { instanceId, listType: activeSubTab, identifier });
      await fetchLists();
    } catch (err) {
      alert(`Error removing player: ${err}`);
    }
  };

  const handleQuickAdd = async (username: string, listType: 'whitelist' | 'ops' | 'banned-players') => {
    try {
      setAdding(true);
      await invoke('add_player', { instanceId, listType, username });
      await fetchLists();
    } catch (err) {
      alert(`Error adding player: ${err}`);
    } finally {
      setAdding(false);
    }
  };

  if (loading && !lists) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
        >
          <Activity className="text-primary w-12 h-12 opacity-50" />
        </motion.div>
        <span className="text-gray-400 dark:text-white/40 font-medium tracking-wider uppercase text-xs">Loading player data...</span>
      </div>
    );
  }

  const subTabs: { id: PlayerSubTab; label: string; icon: any; color: string }[] = [
    { id: 'live', label: 'Live', icon: Activity, color: 'text-emerald-500' },
    { id: 'whitelist', label: 'Whitelist', icon: Shield, color: 'text-primary' },
    { id: 'ops', label: 'Operators', icon: Shield, color: 'text-accent-amber' },
    { id: 'banned-players', label: 'Banned', icon: Ban, color: 'text-accent-rose' },
    { id: 'banned-ips', label: 'Banned IPs', icon: Globe, color: 'text-accent-rose' },
  ];

  const currentList = lists ? (
    activeSubTab === 'whitelist' ? lists.whitelist :
      activeSubTab === 'ops' ? lists.ops :
        activeSubTab === 'banned-players' ? lists.banned_players :
          activeSubTab === 'banned-ips' ? lists.banned_ips :
            []
  ) : [];

  return (
    <div className="flex flex-col h-full space-y-6">
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
        <motion.button
          whileHover={{ scale: 1.02, translateY: -2 }}
          whileTap={{ scale: 0.98 }}
          className="flex items-center gap-2 px-6 py-3 bg-primary/10 text-primary border border-primary/20 rounded-2xl hover:bg-primary/20 transition-all text-xs font-black uppercase tracking-widest"
          onClick={() => invoke('open_instance_folder', { instanceId })}
        >
          <FileText size={18} />
          Edit Raw Configs
        </motion.button>
      </div>

      <AnimatePresence mode="wait">
        {error && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="bg-accent-rose/10 border border-accent-rose/20 text-accent-rose p-5 rounded-2xl flex items-center gap-4"
          >
            <div className="w-10 h-10 rounded-full bg-accent-rose/20 flex items-center justify-center shrink-0">
              <Info size={20} />
            </div>
            <p className="text-sm font-medium">{error}</p>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="flex-1 overflow-y-auto min-h-0 pr-2 custom-scrollbar">
        <AnimatePresence mode="wait">
          <motion.div
            key={activeSubTab}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
          >
            {activeSubTab === 'live' ? (
              onlinePlayers.map((username, index) => (
                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: index * 0.05 }}
                  key={username}
                  className="glass-panel p-4 rounded-2xl flex items-center justify-between border border-black/5 dark:border-white/5 group hover:border-primary/30 transition-all hover:translate-y-[-2px]"
                >
                  <div className="flex items-center gap-4">
                    <div className="relative shrink-0">
                      <img
                        src={`https://minotar.net/avatar/${username}/48`}
                        alt={username}
                        className="w-12 h-12 rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10"
                      />
                      <span className={cn(
                        "absolute -bottom-1 -right-1 w-4 h-4 bg-emerald-500 rounded-full border-2 shadow-glow-emerald",
                        "border-white dark:border-[#0a0a0a]"
                      )}></span>
                    </div>
                    <div className="flex flex-col min-w-0">
                      <span className="font-bold text-gray-900 dark:text-white tracking-tight truncate">{username}</span>
                      <span className="text-[10px] text-emerald-500 font-black uppercase tracking-widest flex items-center gap-1.5">
                        Live Now
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all translate-x-2 group-hover:translate-x-0">
                    <button
                      onClick={() => handleQuickAdd(username, 'whitelist')}
                      title="Add to Whitelist"
                      className="p-2 text-gray-400 dark:text-white/40 hover:text-primary hover:bg-primary/10 rounded-xl transition-colors"
                    >
                      <Shield size={18} />
                    </button>
                    <button
                      onClick={() => handleQuickAdd(username, 'ops')}
                      title="Make Operator"
                      className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-amber hover:bg-accent-amber/10 rounded-xl transition-colors"
                    >
                      <Plus size={18} />
                    </button>
                    <button
                      onClick={() => handleQuickAdd(username, 'banned-players')}
                      title="Ban Player"
                      className="p-2 text-gray-400 dark:text-white/40 hover:text-accent-rose hover:bg-accent-rose/10 rounded-xl transition-colors"
                    >
                      <Ban size={18} />
                    </button>
                  </div>
                </motion.div>
              ))
            ) : (
              currentList.map((player: any, index) => (
                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: index * 0.05 }}
                  key={'uuid' in player ? player.uuid : player.ip}
                  className="glass-panel p-4 rounded-2xl flex items-center justify-between border border-black/5 dark:border-white/5 group hover:border-primary/30 transition-all hover:translate-y-[-2px]"
                >
                  <div className="flex items-center gap-4">
                    {'uuid' in player ? (
                      <img
                        src={`https://minotar.net/avatar/${player.uuid}/48`}
                        alt={player.name}
                        className="w-12 h-12 rounded-xl shadow-lg ring-1 ring-black/10 dark:ring-white/10 shrink-0"
                      />
                    ) : (
                      <div className="w-12 h-12 bg-black/5 dark:bg-white/[0.03] rounded-xl flex items-center justify-center shrink-0 border border-black/10 dark:border-white/5 shadow-lg">
                        <Globe size={24} className="text-gray-400 dark:text-white/20" />
                      </div>
                    )}
                    <div className="flex flex-col min-w-0">
                      <span className="font-bold text-gray-900 dark:text-white tracking-tight truncate">{'name' in player ? player.name : player.ip}</span>
                      {'uuid' in player && (
                        <span className="text-[10px] text-gray-400 dark:text-white/20 font-mono truncate uppercase tracking-tighter">{player.uuid.split('-')[0]}...</span>
                      )}
                    </div>
                  </div>
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={() => handleRemovePlayer('uuid' in player ? player.uuid : player.ip)}
                    className="p-2.5 text-gray-400 dark:text-white/20 hover:text-accent-rose hover:bg-accent-rose/10 rounded-xl transition-all opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0"
                  >
                    <Trash2 size={18} />
                  </motion.button>
                </motion.div>
              ))
            )}
            {((activeSubTab === 'live' && onlinePlayers.length === 0) || (activeSubTab !== 'live' && currentList.length === 0)) && !loading && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="col-span-full py-20 text-center"
              >
                <div className="w-20 h-20 rounded-full bg-black/5 dark:bg-white/[0.03] flex items-center justify-center mx-auto mb-6">
                  {activeSubTab === 'live' ? <Activity size={32} className="text-gray-300 dark:text-white/10" /> : <UserPlus size={32} className="text-gray-300 dark:text-white/10" />}
                </div>
                <h3 className="text-xl font-bold text-gray-400 dark:text-white/40">
                  {activeSubTab === 'live' ? "No players online" : "List is empty"}
                </h3>
                <p className="text-gray-400 dark:text-white/20 mt-2">
                  {activeSubTab === 'live' ? "Invite some friends to join the server!" : "Add a player to get started."}
                </p>
              </motion.div>
            )}
          </motion.div>
        </AnimatePresence>
      </div>

      {activeSubTab !== 'live' && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="pt-6 border-t border-black/5 dark:border-white/5"
        >
          <form onSubmit={handleAddPlayer} className="space-y-3">
            <label className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/30 ml-1">
              {activeSubTab === 'banned-ips' ? 'Ban IP Address' : `Add to ${activeSubTab}`}
            </label>
            <div className="flex gap-3">
              <div className="relative flex-1 group">
                <UserPlus className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={20} />
                <input
                  type="text"
                  placeholder={activeSubTab === 'banned-ips' ? "Enter IP address..." : "Enter Minecraft username..."}
                  className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-2xl py-4 pl-12 pr-6 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20"
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                />
              </div>
              <motion.button
                whileHover={{ scale: 1.02, translateY: -2 }}
                whileTap={{ scale: 0.98 }}
                type="submit"
                disabled={adding || !newUsername.trim()}
                className="px-8 py-4 bg-primary hover:bg-primary-hover disabled:bg-black/5 dark:disabled:bg-white/5 disabled:text-gray-400 dark:disabled:text-white/20 disabled:cursor-not-allowed rounded-2xl transition-all text-sm font-black uppercase tracking-widest text-white shadow-glow-primary flex items-center gap-2 shrink-0"
              >
                {adding ? (
                  <RefreshCw size={18} className="animate-spin" />
                ) : (
                  <Plus size={18} />
                )}
                {adding ? 'Adding...' : 'Add Player'}
              </motion.button>
            </div>
          </form>
        </motion.div>
      )}
    </div>
  );
}
