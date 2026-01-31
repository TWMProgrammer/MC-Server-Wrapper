import { useState, useEffect } from 'react'
import { Trash2, UserPlus, FileText, Shield, Ban, Globe, Activity, Plus } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { AllPlayerLists } from './types'
import { cn } from './utils'

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
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  const subTabs: { id: PlayerSubTab; label: string; icon: any }[] = [
    { id: 'live', label: 'Live', icon: Activity },
    { id: 'whitelist', label: 'Whitelist', icon: Shield },
    { id: 'ops', label: 'Operators', icon: Shield },
    { id: 'banned-players', label: 'Banned', icon: Ban },
    { id: 'banned-ips', label: 'Banned IPs', icon: Globe },
  ];

  const currentList = lists ? (
    activeSubTab === 'whitelist' ? lists.whitelist :
      activeSubTab === 'ops' ? lists.ops :
        activeSubTab === 'banned-players' ? lists.banned_players :
          activeSubTab === 'banned-ips' ? lists.banned_ips :
            []
  ) : [];

  return (
    <div className="flex flex-col h-full space-y-4">
      <div className="flex items-center justify-between border-b border-white/10 pb-4">
        <div className="flex gap-4">
          {subTabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveSubTab(tab.id)}
              className={cn(
                "px-4 py-2 rounded-md transition-colors flex items-center gap-2",
                activeSubTab === tab.id ? "bg-blue-600 text-white" : "text-gray-400 hover:text-white hover:bg-white/5"
              )}
            >
              <tab.icon size={18} />
              {tab.label}
            </button>
          ))}
        </div>
        <button
          className="flex items-center gap-2 px-4 py-2 bg-teal-600/20 text-teal-400 border border-teal-600/50 rounded-md hover:bg-teal-600/30 transition-colors"
          onClick={() => invoke('open_instance_folder', { instanceId })}
        >
          <FileText size={18} />
          Switch to File Editor
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-500/20 border border-red-500/50 text-red-400 rounded-md">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-y-auto min-h-0">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {activeSubTab === 'live' ? (
            onlinePlayers.map((username) => (
              <div
                key={username}
                className="bg-[#2a2a2a] p-3 rounded-lg flex items-center justify-between border border-white/5 group hover:border-white/10 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <img
                    src={`https://minotar.net/avatar/${username}/32`}
                    alt={username}
                    className="w-8 h-8 rounded"
                  />
                  <div className="flex flex-col min-w-0">
                    <span className="font-medium truncate">{username}</span>
                    <span className="text-xs text-green-500 flex items-center gap-1">
                      <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse"></span>
                      Online
                    </span>
                  </div>
                </div>
                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={() => handleQuickAdd(username, 'whitelist')}
                    title="Add to Whitelist"
                    className="p-1.5 text-gray-400 hover:text-blue-400 hover:bg-blue-400/10 rounded transition-colors"
                  >
                    <Shield size={16} />
                  </button>
                  <button
                    onClick={() => handleQuickAdd(username, 'ops')}
                    title="Make Operator"
                    className="p-1.5 text-gray-400 hover:text-yellow-400 hover:bg-yellow-400/10 rounded transition-colors"
                  >
                    <Plus size={16} />
                  </button>
                  <button
                    onClick={() => handleQuickAdd(username, 'banned-players')}
                    title="Ban Player"
                    className="p-1.5 text-gray-400 hover:text-red-400 hover:bg-red-400/10 rounded transition-colors"
                  >
                    <Ban size={16} />
                  </button>
                </div>
              </div>
            ))
          ) : (
            currentList.map((player: any) => (
              <div
                key={'uuid' in player ? player.uuid : player.ip}
                className="bg-[#2a2a2a] p-3 rounded-lg flex items-center justify-between border border-white/5 group hover:border-white/10 transition-colors"
              >
                <div className="flex items-center gap-3">
                  {'uuid' in player ? (
                    <img
                      src={`https://minotar.net/avatar/${player.uuid}/32`}
                      alt={player.name}
                      className="w-8 h-8 rounded"
                    />
                  ) : (
                    <div className="w-8 h-8 bg-gray-700 rounded flex items-center justify-center">
                      <Globe size={16} className="text-gray-400" />
                    </div>
                  )}
                  <div className="flex flex-col min-w-0">
                    <span className="font-medium truncate">{'name' in player ? player.name : player.ip}</span>
                    {'uuid' in player && (
                      <span className="text-xs text-gray-500 truncate">{player.uuid.split('-')[0]}...</span>
                    )}
                  </div>
                </div>
                <button
                  onClick={() => handleRemovePlayer('uuid' in player ? player.uuid : player.ip)}
                  className="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-md transition-all opacity-0 group-hover:opacity-100"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            ))
          )}
          {((activeSubTab === 'live' && onlinePlayers.length === 0) || (activeSubTab !== 'live' && currentList.length === 0)) && !loading && (
            <div className="col-span-full py-12 text-center text-gray-500">
              {activeSubTab === 'live' ? "No players online." : "No entries found in this list."}
            </div>
          )}
        </div>
      </div>

      {activeSubTab !== 'live' && (
        <div className="pt-4 border-t border-white/10">
          <form onSubmit={handleAddPlayer} className="space-y-2">
            <label className="text-sm font-medium text-gray-400">
              {activeSubTab === 'banned-ips' ? 'Add IP Address' : 'Add username'}
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                placeholder={activeSubTab === 'banned-ips' ? "127.0.0.1" : "Steve"}
                className="flex-1 bg-[#121212] border border-white/10 rounded-md px-4 py-2 focus:outline-none focus:border-blue-500 transition-colors"
              />
              <button
                type="submit"
                disabled={adding || !newUsername.trim()}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
              >
                {adding ? (
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                ) : (
                  <UserPlus size={18} />
                )}
                Add
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}
