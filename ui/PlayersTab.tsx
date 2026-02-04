import { useState, useEffect, useMemo, useRef } from 'react'
import { FileText, Shield, Ban, Globe, Activity, Info, Users } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { AllPlayerLists } from './types'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'
import { TextEditor } from './components/TextEditor'
import { PlayerCard } from './components/players/PlayerCard'
import { PlayerListTable } from './components/players/PlayerListTable'
import { PlayerHeader } from './components/players/PlayerHeader'
import { useToast } from './hooks/useToast'
import { AppSettings } from './hooks/useAppSettings'

interface PlayersTabProps {
  instanceId: string;
  settings: AppSettings;
}

export type PlayerSubTab = 'all' | 'whitelist' | 'ops' | 'banned-players' | 'banned-ips';

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05
    }
  }
};

export function PlayersTab({ instanceId, settings }: PlayersTabProps) {
  const [activeSubTab, setActiveSubTab] = useState<PlayerSubTab>('all');
  const [lists, setLists] = useState<AllPlayerLists | null>(null);
  const [onlinePlayers, setOnlinePlayers] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newUsername, setNewUsername] = useState('');
  const [adding, setAdding] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [isRawEditing, setIsRawEditing] = useState(false);
  const [rawContent, setRawContent] = useState('');
  const addModalRef = useRef<HTMLDivElement>(null);
  const { showToast } = useToast();

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (addModalRef.current && !addModalRef.current.contains(event.target as Node)) {
        setIsAddModalOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

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
        showToast(`Banned IP: ${newUsername.trim()}`);
      } else {
        await invoke('add_player', { instanceId, listType: activeSubTab, username: newUsername.trim() });
        showToast(`Added ${newUsername.trim()} to ${activeSubTab}`);
      }
      setNewUsername('');
      setIsAddModalOpen(false);
      await fetchLists();
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    } finally {
      setAdding(false);
    }
  };

  const handleRemovePlayer = async (identifier: string) => {
    try {
      await invoke('remove_player', { instanceId, listType: activeSubTab, identifier });
      showToast(`Removed from ${activeSubTab}`);
      await fetchLists();
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    }
  };

  const handleQuickAdd = async (username: string, listType: 'whitelist' | 'ops' | 'banned-players') => {
    try {
      setAdding(true);
      await invoke('add_player', { instanceId, listType, username });
      showToast(`Added ${username} to ${listType}`);
      await fetchLists();
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    } finally {
      setAdding(false);
    }
  };

  const handleRawEdit = async () => {
    if (activeSubTab === 'all') return;

    const fileName = activeSubTab === 'whitelist' ? 'whitelist.json' :
      activeSubTab === 'ops' ? 'ops.json' :
        activeSubTab === 'banned-players' ? 'banned-players.json' :
          activeSubTab === 'banned-ips' ? 'banned-ips.json' : '';

    if (!fileName) return;

    try {
      const content = await invoke<string>('read_text_file', {
        instanceId,
        relPath: fileName
      });
      setRawContent(content);
      setIsRawEditing(true);
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    }
  };

  const handleRawSave = async (content: string) => {
    const fileName = activeSubTab === 'whitelist' ? 'whitelist.json' :
      activeSubTab === 'ops' ? 'ops.json' :
        activeSubTab === 'banned-players' ? 'banned-players.json' :
          activeSubTab === 'banned-ips' ? 'banned-ips.json' : '';

    if (!fileName) return;

    try {
      await invoke('save_text_file', {
        instanceId,
        relPath: fileName,
        content
      });
      setRawContent(content);
      await fetchLists();
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    }
  };

  const handleOpenExternal = async () => {
    if (activeSubTab === 'all') return;

    const fileName = activeSubTab === 'whitelist' ? 'whitelist.json' :
      activeSubTab === 'ops' ? 'ops.json' :
        activeSubTab === 'banned-players' ? 'banned-players.json' :
          activeSubTab === 'banned-ips' ? 'banned-ips.json' : '';

    if (!fileName) return;

    try {
      await invoke('open_file_in_editor', {
        instanceId,
        relPath: fileName
      });
    } catch (err) {
      showToast(`Error: ${err}`, 'error');
    }
  };

  const allPlayers = useMemo(() => {
    if (!lists) return [];

    const playersMap = new Map<string, {
      name: string;
      uuid?: string;
      isOnline: boolean;
      isWhitelisted: boolean;
      isOp: boolean;
      isBanned: boolean;
    }>();

    const updatePlayer = (name: string, uuid?: string, flags: Partial<{ isOnline: boolean, isWhitelisted: boolean, isOp: boolean, isBanned: boolean }> = {}) => {
      const existing = playersMap.get(name);
      playersMap.set(name, {
        name,
        uuid: uuid || existing?.uuid,
        isOnline: flags.isOnline ?? existing?.isOnline ?? false,
        isWhitelisted: flags.isWhitelisted ?? existing?.isWhitelisted ?? false,
        isOp: flags.isOp ?? existing?.isOp ?? false,
        isBanned: flags.isBanned ?? existing?.isBanned ?? false,
      });
    };

    onlinePlayers.forEach(name => updatePlayer(name, undefined, { isOnline: true }));
    lists.user_cache.forEach(p => updatePlayer(p.name, p.uuid));
    lists.whitelist.forEach(p => updatePlayer(p.name, p.uuid, { isWhitelisted: true }));
    lists.ops.forEach(p => updatePlayer(p.name, p.uuid, { isOp: true }));
    lists.banned_players.forEach(p => updatePlayer(p.name, p.uuid, { isBanned: true }));

    return Array.from(playersMap.values()).sort((a, b) => {
      if (a.isOnline !== b.isOnline) return a.isOnline ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  }, [lists, onlinePlayers]);

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
    { id: 'all', label: 'All Players', icon: Users, color: 'text-emerald-500' },
    { id: 'whitelist', label: 'Whitelist', icon: FileText, color: 'text-primary' },
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
      <PlayerHeader
        activeSubTab={activeSubTab}
        setActiveSubTab={setActiveSubTab}
        subTabs={subTabs}
        isAddModalOpen={isAddModalOpen}
        setIsAddModalOpen={setIsAddModalOpen}
        addModalRef={addModalRef}
        newUsername={newUsername}
        setNewUsername={setNewUsername}
        handleAddPlayer={handleAddPlayer}
        handleRawEdit={handleRawEdit}
        adding={adding}
      />

      <AnimatePresence>
        {isRawEditing && (
          <TextEditor
            title={`Edit ${activeSubTab} list`}
            initialValue={rawContent}
            onSave={handleRawSave}
            onClose={() => setIsRawEditing(false)}
            onOpenExternal={handleOpenExternal}
          />
        )}
      </AnimatePresence>

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
            initial="hidden"
            animate="visible"
            exit={{ opacity: 0, y: -10 }}
            variants={containerVariants}
          >
            {activeSubTab === 'all' ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <AnimatePresence mode="popLayout">
                  {allPlayers.map((player, index) => (
                    <PlayerCard
                      key={player.name}
                      player={player}
                      index={index}
                      onQuickAdd={handleQuickAdd}
                      settings={settings}
                    />
                  ))}
                </AnimatePresence>
              </div>
            ) : (
              <PlayerListTable
                list={currentList}
                onRemove={handleRemovePlayer}
                settings={settings}
              />
            )}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  );
}
