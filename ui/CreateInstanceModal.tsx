import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { X, Search, Filter, Box, Download, Info, Check, ChevronRight, HardDrive, Globe, Package, Zap, Send, Hammer, Layers, Network, Gamepad2, Blocks, Terminal, Sparkles, Plus } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from './utils'
import { Instance } from './types'
import { Select } from './components/Select'

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
    icon: <Box className="text-emerald-400" size={24} />,
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
    description: 'Purpur is a drop-in replacement for Paper servers designed for configurability and new features.',
    category: 'Playable Server',
    icon: <Sparkles className="text-purple-400" size={24} />,
  },
  {
    id: 'forge',
    name: 'Forge',
    description: 'Drastically change the way how Minecraft looks and feels with mods.',
    category: 'Playable Server',
    icon: <Hammer className="text-orange-400" size={24} />,
  },
  {
    id: 'neoforge',
    name: 'NeoForge',
    description: 'A community-driven fork of Forge, designed to be more modern and open.',
    category: 'Playable Server',
    icon: <Zap className="text-amber-400" size={24} />,
  },
  {
    id: 'fabric',
    name: 'Fabric',
    description: 'Fabric is a lightweight, experimental modding toolchain for Minecraft.',
    category: 'Playable Server',
    icon: <Layers className="text-indigo-400" size={24} />,
  },
  {
    id: 'bungeecord',
    name: 'BungeeCord',
    description: 'Efficiently proxies, maintains connections and transport between multiple servers.',
    category: 'Network Proxy',
    icon: <Network className="text-emerald-400" size={24} />,
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
  onCreated: (instance: Instance) => void;
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
    setSelectedVersion(null);
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
    } catch (e) {
      console.error('Failed to load versions', e);
    } finally {
      setLoading(false);
    }
  }

  async function loadModLoaders(version: string) {
    const isModded = ['forge', 'fabric', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '');
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
      const instance = await invoke<Instance>('create_instance_full', {
        name,
        version: selectedVersion,
        modLoader: selectedLoader === 'none' ? null : selectedLoader,
        loaderVersion: selectedLoaderVersion,
      });
      onCreated(instance);
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
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="absolute inset-0 bg-black/80 backdrop-blur-md"
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 20 }}
            className="bg-surface border border-black/10 dark:border-white/10 rounded-3xl shadow-2xl w-full max-w-5xl h-[85vh] flex flex-col overflow-hidden relative z-10 ring-1 ring-black/5 dark:ring-white/5 transition-colors duration-300"
          >
            {/* Header */}
            <div className="p-6 border-b border-black/5 dark:border-white/5 flex items-center justify-between gap-6 bg-black/[0.01] dark:bg-white/[0.02]">
              <div className="flex items-center gap-6 flex-1">
                <div className="w-14 h-14 bg-primary/10 rounded-2xl flex items-center justify-center shadow-glow-primary border border-primary/20">
                  <Plus className="text-primary" size={28} />
                </div>
                <div className="flex-1 max-w-md">
                  <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1.5 ml-1">New Instance</div>
                  <input
                    type="text"
                    placeholder="Enter instance name..."
                    value={name}
                    onChange={e => setName(e.target.value)}
                    className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-5 py-3 text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all font-medium"
                    autoFocus
                  />
                </div>
              </div>
              <motion.button
                whileHover={{ scale: 1.1, rotate: 90 }}
                whileTap={{ scale: 0.9 }}
                onClick={onClose}
                className="p-3 hover:bg-black/5 dark:hover:bg-white/5 rounded-2xl transition-colors text-gray-400 dark:text-white/30 hover:text-gray-900 dark:hover:text-white"
              >
                <X size={24} />
              </motion.button>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex overflow-hidden">
              {/* Sidebar */}
              <div className="w-72 bg-black/5 dark:bg-black/20 border-r border-black/5 dark:border-white/5 p-4 flex flex-col gap-2 transition-colors duration-300">
                <div className="px-4 py-2 text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Sources</div>
                <SidebarItem
                  icon={<Globe size={20} />}
                  label="Official Minecraft"
                  active={activeTab === 'custom'}
                  onClick={() => setActiveTab('custom')}
                />
                <SidebarItem
                  icon={<HardDrive size={20} />}
                  label="Local ZIP File"
                  active={activeTab === 'import'}
                  onClick={() => setActiveTab('import')}
                />
                <div className="my-4 border-t border-black/5 dark:border-white/5" />
                <div className="px-4 py-2 text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Coming Soon</div>
                <SidebarItem
                  icon={<Package size={20} />}
                  label="Modrinth"
                  active={activeTab === 'modrinth'}
                  onClick={() => setActiveTab('modrinth')}
                  disabled
                />
                <SidebarItem
                  icon={<Package size={20} />}
                  label="CurseForge"
                  active={activeTab === 'curseforge'}
                  onClick={() => setActiveTab('curseforge')}
                  disabled
                />
              </div>

              {/* Content Area */}
              <div className="flex-1 flex flex-col overflow-hidden bg-background transition-colors duration-300">
                <AnimatePresence mode="wait">
                  {activeTab === 'custom' ? (
                    <motion.div
                      key="custom"
                      initial={{ opacity: 0, x: 20 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -20 }}
                      className="flex-1 flex flex-col overflow-hidden"
                    >
                      {!selectedServerType ? (
                        /* Software Selection Grid */
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
                                      transition={{ delay: (catIdx * 0.1) + (i * 0.05) }}
                                      whileHover={{ scale: 1.02, translateY: -4 }}
                                      whileTap={{ scale: 0.98 }}
                                      onClick={() => setSelectedServerType(type.id)}
                                      className="flex flex-col gap-4 p-5 rounded-2xl bg-black/[0.02] dark:bg-white/[0.02] border border-black/5 dark:border-white/5 hover:bg-black/[0.04] dark:hover:bg-white/[0.05] hover:border-primary/30 transition-all text-left group relative overflow-hidden"
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
                      ) : (
                        /* Version Selection */
                        <div className="flex-1 flex flex-col overflow-hidden">
                          <div className="p-6 border-b border-black/5 dark:border-white/5 flex items-center justify-between bg-black/[0.01] dark:bg-white/[0.01]">
                            <div className="flex items-center gap-4">
                              <motion.button
                                whileHover={{ scale: 1.1, x: -2 }}
                                whileTap={{ scale: 0.9 }}
                                onClick={() => setSelectedServerType(null)}
                                className="p-2.5 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl text-gray-400 dark:text-white/50 hover:text-gray-900 dark:hover:text-white transition-all border border-black/5 dark:border-white/5"
                              >
                                <ChevronRight size={20} className="rotate-180" />
                              </motion.button>
                              <div>
                                <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1">Software</div>
                                <h3 className="text-lg font-black text-gray-900 dark:text-white flex items-center gap-2">
                                  {SERVER_TYPES.find(t => t.id === selectedServerType)?.name}
                                </h3>
                              </div>
                            </div>

                            <div className="flex items-center gap-4">
                              <div className="relative w-72 group">
                                <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={16} />
                                <input
                                  type="text"
                                  placeholder="Search versions..."
                                  value={search}
                                  onChange={e => setSearch(e.target.value)}
                                  className="w-full bg-black/5 dark:bg-white/5 border border-black/10 dark:border-white/10 rounded-xl pl-12 pr-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all font-medium"
                                />
                              </div>
                              <div className="flex items-center gap-4 text-[11px] font-bold text-gray-500 dark:text-white/40">
                                <label className="flex items-center gap-2.5 cursor-pointer hover:text-gray-900 dark:hover:text-white transition-colors group">
                                  <div className={cn(
                                    "w-5 h-5 rounded-md border-2 flex items-center justify-center transition-all",
                                    showSnapshots ? "bg-primary border-primary shadow-glow-primary" : "border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5"
                                  )}>
                                    {showSnapshots && <Check size={12} className="text-white" />}
                                  </div>
                                  <input
                                    type="checkbox"
                                    checked={showSnapshots}
                                    onChange={e => setShowSnapshots(e.target.checked)}
                                    className="hidden"
                                  />
                                  Snapshots
                                </label>
                              </div>
                            </div>
                          </div>

                          <div className="flex-1 overflow-auto custom-scrollbar">
                            {loading ? (
                              <div className="h-full flex flex-col items-center justify-center gap-4 text-gray-400 dark:text-white/20">
                                <div className="w-12 h-12 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
                                <span className="text-sm font-bold tracking-widest uppercase">Fetching versions...</span>
                              </div>
                            ) : (
                              <table className="w-full text-left text-sm border-separate border-spacing-y-2 px-6">
                                <thead className="sticky top-0 bg-background/80 backdrop-blur-md z-10 transition-colors duration-300">
                                  <tr className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">
                                    <th className="px-6 py-4">Version</th>
                                    <th className="px-6 py-4">Type</th>
                                    <th className="px-6 py-4">Release Date</th>
                                  </tr>
                                </thead>
                                <tbody>
                                  {filteredVersions.map(v => (
                                    <motion.tr
                                      key={v.id}
                                      initial={{ opacity: 0, y: 10 }}
                                      animate={{ opacity: 1, y: 0 }}
                                      onClick={() => setSelectedVersion(v.id)}
                                      className={cn(
                                        "cursor-pointer transition-all group",
                                        selectedVersion === v.id
                                          ? "bg-primary/10 text-gray-900 dark:text-white"
                                          : "text-gray-600 dark:text-white/50 hover:bg-black/[0.02] dark:hover:bg-white/[0.03]"
                                      )}
                                    >
                                      <td className="px-6 py-4 first:rounded-l-2xl border-y border-l border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                                        <div className="flex items-center gap-3">
                                          <div className={cn(
                                            "w-6 h-6 rounded-full flex items-center justify-center transition-all",
                                            selectedVersion === v.id ? "bg-primary text-white scale-110 shadow-glow-primary" : "bg-black/5 dark:bg-white/5 text-transparent border border-black/10 dark:border-white/10"
                                          )}>
                                            <Check size={14} />
                                          </div>
                                          <span className="font-bold font-mono tracking-tight">{v.id}</span>
                                        </div>
                                      </td>
                                      <td className="px-6 py-4 border-y border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                                        <span className={cn(
                                          "px-2.5 py-1 rounded-lg text-[10px] font-black uppercase tracking-wider",
                                          v.type === 'release' ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400" : "bg-amber-500/10 text-amber-600 dark:text-amber-400"
                                        )}>
                                          {v.type.replace('_', ' ')}
                                        </span>
                                      </td>
                                      <td className="px-6 py-4 last:rounded-r-2xl border-y border-r border-black/5 dark:border-white/5 group-hover:border-primary/20 transition-colors">
                                        <span className="text-gray-400 dark:text-white/20 font-medium group-hover:text-gray-900 dark:group-hover:text-white/40 transition-colors">
                                          {new Date(v.releaseTime).toLocaleDateString(undefined, { dateStyle: 'long' })}
                                        </span>
                                      </td>
                                    </motion.tr>
                                  ))}
                                </tbody>
                              </table>
                            )}
                          </div>

                          {/* Mod Loader Selection */}
                          {['forge', 'fabric', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '') && (
                            <motion.div
                              initial={{ y: 20, opacity: 0 }}
                              animate={{ y: 0, opacity: 1 }}
                              className="p-6 border-t border-black/5 dark:border-white/5 bg-black/[0.01] dark:bg-white/[0.02] flex items-center justify-between transition-colors duration-300"
                            >
                              <div className="flex items-center gap-4">
                                <div className="p-3 rounded-xl bg-primary/10 text-primary">
                                  <Terminal size={20} />
                                </div>
                                <div>
                                  <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-0.5">Software Build</div>
                                  <div className="text-sm font-bold text-gray-600 dark:text-white/70">
                                    Select {SERVER_TYPES.find(t => t.id === selectedServerType)?.name} version
                                  </div>
                                </div>
                              </div>
                              <div className="flex items-center gap-3 min-w-[200px]">
                                <Select
                                  value={selectedLoaderVersion || ''}
                                  onChange={newValue => setSelectedLoaderVersion(newValue)}
                                  options={
                                    modLoaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase()))?.versions.map(v => ({
                                      value: v,
                                      label: v
                                    })) || []
                                  }
                                  placeholder="Select version"
                                />
                              </div>
                            </motion.div>
                          )}
                        </div>
                      )}
                    </motion.div>
                  ) : (
                    <motion.div
                      key="soon"
                      initial={{ opacity: 0, scale: 0.95 }}
                      animate={{ opacity: 1, scale: 1 }}
                      exit={{ opacity: 0, scale: 0.95 }}
                      className="flex-1 flex flex-col items-center justify-center text-gray-400 dark:text-white/10 gap-6 p-12 text-center"
                    >
                      <div className="w-24 h-24 rounded-full bg-black/5 dark:bg-white/[0.02] border border-black/5 dark:border-white/5 flex items-center justify-center">
                        <Box size={48} strokeWidth={1} />
                      </div>
                      <div className="max-w-xs space-y-2">
                        <h3 className="text-lg font-black text-gray-400 dark:text-white/30 uppercase tracking-[0.2em]">Coming Soon</h3>
                        <p className="text-sm font-medium leading-relaxed text-gray-500 dark:text-white/40">We're working hard to bring {activeTab === 'import' ? 'ZIP imports' : activeTab} to the wrapper!</p>
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            </div>

            {/* Footer */}
            <div className="p-6 border-t border-black/5 dark:border-white/5 flex items-center justify-between bg-black/5 dark:bg-black/40 backdrop-blur-xl transition-colors duration-300">
              <div className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20 flex items-center gap-3">
                <Info size={16} className="text-primary" />
                <span>{selectedVersion ? `Ready to install Minecraft ${selectedVersion}` : 'Select a software and version to continue'}</span>
              </div>
              <div className="flex items-center gap-4">
                <button
                  onClick={onClose}
                  className="px-8 py-3 rounded-xl text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-all"
                >
                  Cancel
                </button>
                <motion.button
                  whileHover={(!name || !selectedVersion || creating) ? {} : { scale: 1.02, translateY: -2 }}
                  whileTap={(!name || !selectedVersion || creating) ? {} : { scale: 0.98 }}
                  onClick={handleCreate}
                  disabled={!name || !selectedVersion || creating}
                  className={cn(
                    "px-10 py-3 rounded-xl text-xs font-black uppercase tracking-widest transition-all flex items-center gap-3 shadow-2xl",
                    !name || !selectedVersion || creating
                      ? "bg-black/5 dark:bg-white/5 text-gray-400 dark:text-white/10 cursor-not-allowed"
                      : "bg-primary hover:bg-primary-hover text-white shadow-glow-primary"
                  )}
                >
                  {creating ? (
                    <>
                      <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" />
                      Creating...
                    </>
                  ) : (
                    <>
                      <Plus size={18} />
                      Create Instance
                    </>
                  )}
                </motion.button>
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
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
          ? "bg-primary text-white shadow-lg shadow-primary/20"
          : disabled
            ? "text-gray-400 dark:text-white/10 cursor-not-allowed"
            : "text-gray-500 dark:text-white/50 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5"
      )}
    >
      {icon}
      {label}
      {active && <ChevronRight size={14} className="ml-auto opacity-50" />}
    </button>
  )
}
