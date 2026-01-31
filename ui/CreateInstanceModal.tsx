import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { X, Search, Filter, Box, Download, Info, Check, ChevronRight, HardDrive, Globe, Package, Zap, Send, Hammer, Layers, Network, Gamepad2, Blocks } from 'lucide-react'

interface ServerType {
  id: string;
  name: string;
  description: string;
  category: 'Playable Server' | 'Network Proxy' | 'Other';
  icon: React.ReactNode;
  badge?: string;
  badgeColor?: string;
}

const SERVER_TYPES: ServerType[] = [
  {
    id: 'vanilla',
    name: 'Vanilla',
    description: 'The basic Vanilla experience without plugins.',
    category: 'Playable Server',
    icon: <Box className="text-green-500" size={24} />,
  },
  {
    id: 'paper',
    name: 'Paper',
    description: 'High performance fork of Spigot with many features and performance improvements.',
    category: 'Playable Server',
    icon: <Send className="text-blue-400" size={24} />,
  },
  {
    id: 'purpur',
    name: 'Purpur',
    description: 'Purpur is a drop-in replacement for Paper servers designed for configurability and new, fun, exciting gameplay features.',
    category: 'Playable Server',
    icon: <Box className="text-purple-500" size={24} />,
  },
  {
    id: 'forge',
    name: 'Forge',
    description: 'Drastically change the way how Minecraft looks and feels with mods.',
    category: 'Playable Server',
    icon: <Hammer className="text-orange-500" size={24} />,
  },
  {
    id: 'neoforge',
    name: 'NeoForge',
    description: 'A community-driven fork of Forge, designed to be more modern and open.',
    category: 'Playable Server',
    icon: <Zap className="text-orange-400" size={24} />,
  },
  {
    id: 'fabric',
    name: 'Fabric',
    description: 'Fabric is a lightweight, experimental modding toolchain for Minecraft.',
    category: 'Playable Server',
    icon: <Layers className="text-orange-200" size={24} />,
  },
  {
    id: 'quilt',
    name: 'Quilt',
    description: 'The Quilt Project is an open-source, community-driven modding toolchain.',
    category: 'Playable Server',
    icon: <Layers className="text-purple-400" size={24} />,
  },
  {
    id: 'bungeecord',
    name: 'BungeeCord',
    description: 'Efficiently proxies, maintains connections and transport between multiple servers.',
    category: 'Network Proxy',
    icon: <Network className="text-green-500" size={24} />,
  },
  {
    id: 'velocity',
    name: 'Velocity',
    description: 'Modern alternative to Waterfall. Designed with performance and stability in mind.',
    category: 'Network Proxy',
    icon: <Zap className="text-blue-400" size={24} />,
  },
  {
    id: 'bedrock',
    name: 'Bedrock',
    description: 'Multi-platform versions of Minecraft for Mobile, Console & Other',
    category: 'Other',
    icon: <Gamepad2 className="text-gray-400" size={24} />,
    badge: 'preview',
    badgeColor: 'text-orange-400',
  },
];
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface MCVersion {
  id: string;
  type: string;
  url: string;
  releaseTime: string;
}

interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: MCVersion[];
}

interface ModLoader {
  name: string;
  versions: string[];
}

interface CreateInstanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreated: () => void;
}

type Tab = 'custom' | 'import' | 'modrinth' | 'curseforge';

export function CreateInstanceModal({ isOpen, onClose, onCreated }: CreateInstanceModalProps) {
  const [activeTab, setActiveTab] = useState<Tab>('custom');
  const [selectedServerType, setSelectedServerType] = useState<string | null>(null);
  const [manifest, setManifest] = useState<VersionManifest | null>(null);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [showSnapshots, setShowSnapshots] = useState(false);
  const [showBetas, setShowBetas] = useState(false);

  const [name, setName] = useState('');
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [modLoaders, setModLoaders] = useState<ModLoader[]>([]);
  const [selectedLoader, setSelectedLoader] = useState<string>('none');
  const [selectedLoaderVersion, setSelectedLoaderVersion] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const resetForm = () => {
    setActiveTab('custom');
    setSelectedServerType(null);
    setSearch('');
    setName('');
    setShowSnapshots(false);
    setShowBetas(false);
    if (manifest?.latest?.release) {
      setSelectedVersion(manifest.latest.release);
    }
    setSelectedLoader('none');
    setSelectedLoaderVersion(null);
  };

  useEffect(() => {
    if (isOpen) {
      loadVersions();
    }
  }, [isOpen]);

  useEffect(() => {
    if (selectedVersion) {
      loadModLoaders(selectedVersion);
    } else {
      setModLoaders([]);
      setSelectedLoader('none');
      setSelectedLoaderVersion(null);
    }
  }, [selectedVersion]);

  useEffect(() => {
    // Automatically set the loader based on server type
    if (selectedServerType === 'forge') {
      setSelectedLoader('forge');
    } else if (selectedServerType === 'fabric') {
      setSelectedLoader('fabric');
    } else if (selectedServerType === 'vanilla') {
      setSelectedLoader('none');
    } else if (selectedServerType) {
      // For others like Paper, Spigot, Purpur, etc.
      setSelectedLoader(selectedServerType);
    } else {
      setSelectedLoader('none');
    }
  }, [selectedServerType]);

  async function loadVersions() {
    try {
      setLoading(true);
      const m = await invoke<VersionManifest>('get_minecraft_versions');
      setManifest(m);
      if (m.latest.release) {
        setSelectedVersion(m.latest.release);
      }
    } catch (e) {
      console.error('Failed to load versions', e);
    } finally {
      setLoading(false);
    }
  }

  async function loadModLoaders(version: string) {
    const isModded = ['forge', 'fabric', 'quilt', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '');
    if (!isModded) {
      setModLoaders([]);
      return;
    }

    try {
      const loaders = await invoke<ModLoader[]>('get_mod_loaders', { mcVersion: version });
      setModLoaders(loaders);
      // Set default loader version if available
      const currentLoader = loaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase()));
      if (currentLoader && currentLoader.versions.length > 0) {
        setSelectedLoaderVersion(currentLoader.versions[0]);
      }
    } catch (e) {
      console.error('Failed to load mod loaders', e);
    }
  }

  const filteredVersions = useMemo(() => {
    if (!manifest) return [];
    return manifest.versions.filter(v => {
      const matchesSearch = v.id.toLowerCase().includes(search.toLowerCase());
      const isRelease = v.type === 'release';
      const isSnapshot = v.type === 'snapshot';

      if (!matchesSearch) return false;
      if (isRelease) return true;
      if (isSnapshot && showSnapshots) return true;
      return false;
    });
  }, [manifest, search, showSnapshots]);

  async function handleCreate() {
    if (!name || !selectedVersion) return;

    try {
      setCreating(true);
      await invoke('create_instance_full', {
        name,
        version: selectedVersion,
        modLoader: selectedLoader === 'none' ? null : selectedLoader,
        loaderVersion: selectedLoaderVersion,
      });
      onCreated();
      resetForm();
      onClose();
    } catch (e) {
      console.error('Failed to create instance', e);
    } finally {
      setCreating(false);
    }
  }

  if (!isOpen) return null;

  const categories = ['Playable Server', 'Network Proxy', 'Other'] as const;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="bg-[#1a1b1e] border border-white/10 rounded-xl shadow-2xl w-full max-w-5xl h-[80vh] flex flex-col overflow-hidden">
        {/* Header */}
        <div className="p-6 border-b border-white/10 flex items-center justify-between gap-4">
          <div className="flex items-center gap-4 flex-1">
            <div className="w-12 h-12 bg-green-600 rounded-lg flex items-center justify-center shadow-lg shadow-green-900/20">
              <Box className="text-white" size={24} />
            </div>
            <div className="flex-1 max-w-md">
              <input
                type="text"
                placeholder="Instance Name"
                value={name}
                onChange={e => setName(e.target.value)}
                className="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                autoFocus
              />
            </div>
          </div>
          <button onClick={onClose} className="p-2 hover:bg-white/5 rounded-lg transition-colors text-white/50 hover:text-white">
            <X size={20} />
          </button>
        </div>

        {/* Main Content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Sidebar */}
          <div className="w-64 bg-black/20 border-r border-white/10 p-2 flex flex-col gap-1">
            <SidebarItem
              icon={<Globe size={18} />}
              label="Minecraft"
              active={activeTab === 'custom'}
              onClick={() => setActiveTab('custom')}
            />
            <SidebarItem
              icon={<HardDrive size={18} />}
              label="Import from ZIP"
              active={activeTab === 'import'}
              onClick={() => setActiveTab('import')}
            />
            <div className="my-2 border-t border-white/5" />
            <SidebarItem
              icon={<Package size={18} />}
              label="Modrinth"
              active={activeTab === 'modrinth'}
              onClick={() => setActiveTab('modrinth')}
              disabled
            />
            <SidebarItem
              icon={<Package size={18} />}
              label="CurseForge"
              active={activeTab === 'curseforge'}
              onClick={() => setActiveTab('curseforge')}
              disabled
            />
          </div>

          {/* Content Area */}
          <div className="flex-1 flex flex-col overflow-hidden bg-[#121214]">
            {activeTab === 'custom' && (
              <div className="flex-1 flex flex-col overflow-hidden">
                {!selectedServerType ? (
                  /* Software Selection Grid */
                  <div className="flex-1 overflow-auto p-6">
                    <div className="space-y-8">
                      {categories.map(category => (
                        <div key={category} className="space-y-4">
                          <h2 className="text-lg font-bold text-white/90 px-1">{category}</h2>
                          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {SERVER_TYPES.filter(t => t.category === category).map(type => (
                              <button
                                key={type.id}
                                onClick={() => setSelectedServerType(type.id)}
                                className="flex items-start gap-4 p-4 rounded-xl bg-white/[0.03] border border-white/5 hover:bg-white/[0.07] hover:border-white/10 transition-all text-left group"
                              >
                                <div className="p-3 rounded-lg bg-black/20 group-hover:bg-black/40 transition-colors">
                                  {type.icon}
                                </div>
                                <div className="flex-1 space-y-1">
                                  <div className="flex items-center gap-2">
                                    <span className="font-bold text-white group-hover:text-blue-400 transition-colors">{type.name}</span>
                                    {type.badge && (
                                      <span className={cn("text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-white/5", type.badgeColor)}>
                                        ({type.badge})
                                      </span>
                                    )}
                                  </div>
                                  <p className="text-xs text-white/50 leading-relaxed line-clamp-2">
                                    {type.description}
                                  </p>
                                </div>
                              </button>
                            ))}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : (
                  /* Version Selection (Current UI) */
                  <div className="flex-1 flex flex-col overflow-hidden">
                    <div className="p-4 border-b border-white/10 flex items-center justify-between bg-white/[0.02]">
                      <div className="flex items-center gap-3">
                        <button
                          onClick={() => setSelectedServerType(null)}
                          className="p-2 hover:bg-white/5 rounded-lg text-white/50 hover:text-white transition-colors"
                        >
                          <ChevronRight size={18} className="rotate-180" />
                        </button>
                        <div>
                          <h3 className="text-sm font-bold text-white flex items-center gap-2">
                            {SERVER_TYPES.find(t => t.id === selectedServerType)?.name}
                            <span className="text-white/30 font-normal">/</span>
                            Select Version
                          </h3>
                        </div>
                      </div>

                      <div className="flex items-center gap-4">
                        <div className="relative w-64">
                          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-white/30" size={14} />
                          <input
                            type="text"
                            placeholder="Search versions..."
                            value={search}
                            onChange={e => setSearch(e.target.value)}
                            className="w-full bg-white/5 border border-white/10 rounded-lg pl-9 pr-4 py-1.5 text-xs text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                          />
                        </div>
                        <div className="flex items-center gap-3 text-xs text-white/70">
                          <label className="flex items-center gap-2 cursor-pointer hover:text-white transition-colors">
                            <input
                              type="checkbox"
                              checked={showSnapshots}
                              onChange={e => setShowSnapshots(e.target.checked)}
                              className="rounded border-white/10 bg-white/5 text-blue-500 focus:ring-0"
                            />
                            Snapshots
                          </label>
                        </div>
                      </div>
                    </div>

                    <div className="flex-1 overflow-auto p-2">
                      {loading ? (
                        <div className="h-full flex flex-col items-center justify-center gap-3 text-white/30">
                          <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                          <span className="text-sm">Fetching versions...</span>
                        </div>
                      ) : (
                        <table className="w-full text-left text-sm">
                          <thead className="sticky top-0 bg-[#121214] z-10 text-white/40 font-medium">
                            <tr>
                              <th className="px-4 py-2">Version</th>
                              <th className="px-4 py-2">Type</th>
                              <th className="px-4 py-2">Release Date</th>
                            </tr>
                          </thead>
                          <tbody className="divide-y divide-white/5">
                            {filteredVersions.map(v => (
                              <tr
                                key={v.id}
                                onClick={() => setSelectedVersion(v.id)}
                                className={cn(
                                  "cursor-pointer transition-colors",
                                  selectedVersion === v.id ? "bg-blue-500/20 text-white" : "text-white/70 hover:bg-white/5"
                                )}
                              >
                                <td className="px-4 py-3 font-medium flex items-center gap-2">
                                  {selectedVersion === v.id && <Check size={14} className="text-blue-400" />}
                                  {v.id}
                                </td>
                                <td className="px-4 py-3 capitalize">{v.type.replace('_', ' ')}</td>
                                <td className="px-4 py-3 text-white/40">
                                  {new Date(v.releaseTime).toLocaleDateString()}
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      )}
                    </div>

                    {/* Mod Loader Version Selection (if applicable) */}
                    {['forge', 'fabric', 'quilt', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '') && (
                      <div className="p-4 border-t border-white/10 bg-white/[0.02]">
                        <div className="flex items-center gap-4">
                          <div className="text-sm font-medium text-white/70">
                            {SERVER_TYPES.find(t => t.id === selectedServerType)?.name} Version:
                          </div>
                          <div className="flex-1 flex items-center gap-2">
                            <select
                              value={selectedLoaderVersion || ''}
                              onChange={e => setSelectedLoaderVersion(e.target.value)}
                              className="bg-[#1a1b1e] border border-white/10 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                            >
                              {modLoaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase()))?.versions.map(v => (
                                <option key={v} value={v}>{v}</option>
                              ))}
                              {!modLoaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase())) && (
                                <option disabled>No versions available</option>
                              )}
                            </select>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}

            {activeTab !== 'custom' && (
              <div className="flex-1 flex flex-col items-center justify-center text-white/20 gap-4">
                <Box size={48} strokeWidth={1} />
                <p className="text-sm">This source is coming soon!</p>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-white/10 flex items-center justify-between bg-black/40">
          <div className="text-xs text-white/30 flex items-center gap-2">
            <Info size={14} />
            <span>Select a version and name your instance to continue</span>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-6 py-2 rounded-lg text-sm font-medium text-white/50 hover:text-white hover:bg-white/5 transition-all"
            >
              Cancel
            </button>
            <button
              onClick={handleCreate}
              disabled={!name || !selectedVersion || creating}
              className={cn(
                "px-8 py-2 rounded-lg text-sm font-bold transition-all flex items-center gap-2 shadow-lg",
                !name || !selectedVersion || creating
                  ? "bg-white/5 text-white/20 cursor-not-allowed"
                  : "bg-blue-600 hover:bg-blue-500 text-white shadow-blue-900/20"
              )}
            >
              {creating ? (
                <>
                  <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  Creating...
                </>
              ) : (
                <>
                  <Download size={16} />
                  Create Instance
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

function SidebarItem({ icon, label, active, onClick, disabled = false }: {
  icon: React.ReactNode,
  label: string,
  active: boolean,
  onClick: () => void,
  disabled?: boolean
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all",
        active
          ? "bg-blue-600 text-white shadow-lg shadow-blue-900/20"
          : disabled
            ? "text-white/10 cursor-not-allowed"
            : "text-white/50 hover:text-white hover:bg-white/5"
      )}
    >
      {icon}
      {label}
      {active && <ChevronRight size={14} className="ml-auto opacity-50" />}
    </button>
  )
}
