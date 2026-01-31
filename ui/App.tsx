import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  Play,
  Square,
  Settings,
  Plus,
  LayoutDashboard,
  Database,
  Download,
  Activity,
  X,
  Network,
  Beaker,
  Users,
  Tag,
  FolderOpen,
  Terminal,
  Puzzle,
  History,
  Calendar
} from 'lucide-react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from 'recharts'
import { CreateInstanceModal } from './CreateInstanceModal'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface Instance {
  id: string;
  name: string;
  version: string;
  path: string;
  created_at: string;
  last_run?: string;
  server_type?: string;
  ip?: string;
  port?: number;
  description?: string;
  max_players?: number;
}

interface ResourceUsage {
  cpu_usage: number;
  memory_usage: number;
  timestamp?: number;
}

type TabId = 'dashboard' | 'console' | 'plugins' | 'players' | 'backups' | 'scheduler' | 'settings';

function App() {
  const [instances, setInstances] = useState<Instance[]>([])
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<TabId>('dashboard')
  const [status, setStatus] = useState<string>('Stopped')
  const [usage, setUsage] = useState<ResourceUsage | null>(null)
  const [history, setHistory] = useState<ResourceUsage[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [logs, setLogs] = useState<Record<string, string[]>>({})
  const [command, setCommand] = useState('')
  const historyRef = useRef<ResourceUsage[]>([])
  const consoleEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!(window as any).__TAURI_INTERNALS__) {
      setLoading(false);
      return;
    }
    loadInstances()

    const unlisten = listen<{ instance_id: string, line: string }>('server-log', (event) => {
      setLogs(prev => ({
        ...prev,
        [event.payload.instance_id]: [...(prev[event.payload.instance_id] || []), event.payload.line].slice(-500)
      }))
    })

    return () => {
      unlisten.then(f => f())
    }
  }, [])

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [logs, selectedInstance])

  useEffect(() => {
    let interval: number;
    if (selectedInstance && (window as any).__TAURI_INTERNALS__) {
      setHistory([])
      historyRef.current = []
      interval = window.setInterval(async () => {
        try {
          const s = await invoke<string>('get_server_status', { instanceId: selectedInstance })
          setStatus(s)
          if (s === 'Running') {
            const u = await invoke<ResourceUsage>('get_server_usage', { instanceId: selectedInstance })
            const usageWithTime = { ...u, timestamp: Date.now() }
            setUsage(u)

            const newHistory = [...historyRef.current, usageWithTime].slice(-30)
            historyRef.current = newHistory
            setHistory(newHistory)
          } else {
            setUsage(null)
          }
        } catch (e) {
          console.error(e)
        }
      }, 2000)
    }
    return () => clearInterval(interval)
  }, [selectedInstance])

  async function loadInstances() {
    if (!(window as any).__TAURI_INTERNALS__) return;
    try {
      const list = await invoke<Instance[]>('list_instances')
      const enrichedList = list.map(inst => ({
        ...inst,
        server_type: inst.server_type || 'Paper',
        ip: inst.ip || '127.0.0.1',
        port: inst.port || 25565,
        description: inst.description || 'There is no description for this server.',
        max_players: inst.max_players || 20
      }))
      setInstances(enrichedList)
      setLoading(false)
    } catch (e) {
      console.error(e)
      setLoading(false)
    }
  }

  async function startServer() {
    if (!selectedInstance || !(window as any).__TAURI_INTERNALS__) return;
    try {
      await invoke('start_server', { instanceId: selectedInstance })
    } catch (e) {
      console.error(e)
    }
  }

  async function stopServer() {
    if (!selectedInstance || !(window as any).__TAURI_INTERNALS__) return;
    try {
      await invoke('stop_server', { instanceId: selectedInstance })
    } catch (e) {
      console.error(e)
    }
  }

  async function handleSendCommand(e: React.FormEvent) {
    e.preventDefault()
    if (!command || !selectedInstance) return
    try {
      await invoke('send_command', { instanceId: selectedInstance, command })
      setCommand('')
    } catch (e) {
      console.error(e)
    }
  }

  const currentInstance = instances.find(i => i.id === selectedInstance);

  const TABS: { id: TabId; label: string; icon: any }[] = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'console', label: 'Console', icon: Terminal },
    { id: 'plugins', label: 'Plugins', icon: Puzzle },
    { id: 'players', label: 'Players', icon: Users },
    { id: 'backups', label: 'Backups', icon: History },
    { id: 'scheduler', label: 'Scheduler', icon: Calendar },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  return (
    <div className="flex h-screen bg-[#1a1a1a] text-white">
      {/* Sidebar */}
      <div className="w-64 bg-[#242424] border-r border-white/10 flex flex-col">
        <div className="p-4 border-b border-white/10 flex items-center gap-2">
          <Database className="text-green-500" />
          <h1 className="font-bold text-lg">MC Wrapper</h1>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          <div className="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">
            Instances
          </div>
          {instances.map(inst => (
            <button
              key={inst.id}
              onClick={() => setSelectedInstance(inst.id)}
              className={cn(
                "w-full text-left px-3 py-2 rounded transition-colors flex items-center gap-2",
                selectedInstance === inst.id ? "bg-green-600 text-white" : "hover:bg-white/5 text-gray-300"
              )}
            >
              <div className={cn("w-2 h-2 rounded-full", selectedInstance === inst.id ? "bg-white" : "bg-gray-500")} />
              <span className="truncate">{inst.name}</span>
            </button>
          ))}
          <button
            onClick={() => setShowCreateModal(true)}
            className="w-full text-left px-3 py-2 rounded hover:bg-white/5 text-green-500 flex items-center gap-2 mt-2"
          >
            <Plus size={18} />
            <span>Create New</span>
          </button>
        </div>

        <div className="p-4 border-t border-white/10 text-xs text-gray-500">
          v0.1.0-alpha
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden bg-[#1e1e1e]">
        {selectedInstance && currentInstance ? (
          <>
            {/* Header */}
            <div className="px-6 pt-6 pb-2 bg-[#242424]">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 bg-green-600 rounded-lg flex items-center justify-center shadow-lg">
                    <Database size={24} />
                  </div>
                  <div>
                    <div className="flex items-center gap-3">
                      <h2 className="text-2xl font-bold">{currentInstance.name}</h2>
                      <div className="flex items-center gap-1.5">
                        <div className={cn(
                          "w-2 h-2 rounded-full",
                          status === 'Running' ? "bg-green-500" :
                            status === 'Starting' ? "bg-yellow-500" : "bg-red-500"
                        )} />
                        <span className={cn(
                          "text-sm font-medium",
                          status === 'Running' ? "text-green-400" :
                            status === 'Starting' ? "text-yellow-400" : "text-red-400"
                        )}>
                          {status === 'Stopped' ? 'Offline' : status}
                        </span>
                      </div>
                    </div>

                    <div className="flex items-center gap-6 mt-2 text-gray-400">
                      <div className="flex items-center gap-1.5 text-sm">
                        <Network size={14} />
                        <span>{currentInstance.ip}:{currentInstance.port}</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Beaker size={14} />
                        <span>{currentInstance.server_type} {currentInstance.version}</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Users size={14} />
                        <span>0/{currentInstance.max_players} players</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Tag size={14} />
                        <span>{currentInstance.description}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div>
                  {status === 'Stopped' || status === 'Crashed' ? (
                    <button
                      onClick={startServer}
                      className="flex items-center gap-2 px-6 py-2 bg-[#2d333b] hover:bg-[#343b44] text-green-500 rounded-md font-bold transition-colors border border-white/5"
                    >
                      <Play size={18} fill="currentColor" />
                      Start
                    </button>
                  ) : (
                    <button
                      onClick={stopServer}
                      className="flex items-center gap-2 px-6 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md font-bold transition-colors shadow-lg"
                    >
                      <Square size={18} fill="currentColor" />
                      Stop
                    </button>
                  )}
                </div>
              </div>

              {/* Tabs */}
              <div className="flex items-center justify-between border-t border-white/5 mt-4">
                <div className="flex gap-1">
                  {TABS.map(tab => (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={cn(
                        "px-4 py-3 text-sm font-medium transition-all relative",
                        activeTab === tab.id ? "text-white" : "text-gray-500 hover:text-gray-300"
                      )}
                    >
                      {tab.label}
                      {activeTab === tab.id && (
                        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-green-500" />
                      )}
                    </button>
                  ))}
                </div>
                <div className="flex items-center gap-4 text-gray-500">
                  <button className="hover:text-white transition-colors">
                    <FolderOpen size={20} />
                  </button>
                  <button className="hover:text-white transition-colors">
                    <Settings size={20} />
                  </button>
                </div>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-6">
              {activeTab === 'dashboard' && (
                <div className="space-y-6">
                  {/* Dashboard Grid */}
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="text-gray-400 text-sm mb-1">CPU Usage</div>
                      <div className="text-2xl font-mono">{usage?.cpu_usage.toFixed(1) || '0.0'}%</div>
                    </div>
                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="text-gray-400 text-sm mb-1">Memory Usage</div>
                      <div className="text-2xl font-mono">{(usage?.memory_usage || 0) / 1024 / 1024 > 1024 ? `${((usage?.memory_usage || 0) / 1024 / 1024 / 1024).toFixed(2)} GB` : `${((usage?.memory_usage || 0) / 1024 / 1024).toFixed(0)} MB`}</div>
                    </div>
                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="text-gray-400 text-sm mb-1">Players</div>
                      <div className="text-2xl font-mono">0 / {currentInstance.max_players}</div>
                    </div>
                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="text-gray-400 text-sm mb-1">Uptime</div>
                      <div className="text-2xl font-mono">00:00:00</div>
                    </div>
                  </div>

                  {/* Charts Section */}
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="flex items-center gap-2 mb-4 text-sm font-medium text-gray-400">
                        <Activity size={16} />
                        CPU Usage History
                      </div>
                      <div className="h-48 w-full">
                        <ResponsiveContainer width="100%" height="100%">
                          <AreaChart data={history}>
                            <defs>
                              <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3} />
                                <stop offset="95%" stopColor="#22c55e" stopOpacity={0} />
                              </linearGradient>
                            </defs>
                            <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                            <XAxis dataKey="timestamp" hide />
                            <YAxis domain={[0, 100]} hide />
                            <Tooltip
                              contentStyle={{ backgroundColor: '#1a1a1a', border: '1px solid #333' }}
                              labelStyle={{ display: 'none' }}
                            />
                            <Area type="monotone" dataKey="cpu_usage" stroke="#22c55e" fillOpacity={1} fill="url(#colorCpu)" isAnimationActive={false} />
                          </AreaChart>
                        </ResponsiveContainer>
                      </div>
                    </div>

                    <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
                      <div className="flex items-center gap-2 mb-4 text-sm font-medium text-gray-400">
                        <Activity size={16} />
                        Memory Usage History
                      </div>
                      <div className="h-48 w-full">
                        <ResponsiveContainer width="100%" height="100%">
                          <AreaChart data={history}>
                            <defs>
                              <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                              </linearGradient>
                            </defs>
                            <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                            <XAxis dataKey="timestamp" hide />
                            <YAxis hide />
                            <Tooltip
                              contentStyle={{ backgroundColor: '#1a1a1a', border: '1px solid #333' }}
                              labelStyle={{ display: 'none' }}
                              formatter={(value: number | undefined) => {
                                if (value === undefined) return ['0 MB', 'Memory'];
                                return [`${(value / 1024 / 1024).toFixed(0)} MB`, 'Memory'];
                              }}
                            />
                            <Area type="monotone" dataKey="memory_usage" stroke="#3b82f6" fillOpacity={1} fill="url(#colorMem)" isAnimationActive={false} />
                          </AreaChart>
                        </ResponsiveContainer>
                      </div>
                    </div>
                  </div>

                  {/* Console Preview */}
                  <div className="bg-[#000] rounded-lg border border-white/5 flex flex-col h-80 overflow-hidden">
                    <div className="bg-[#242424] px-4 py-2 border-b border-white/5 text-sm font-medium flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <Terminal size={16} />
                        Console
                      </div>
                      <button
                        onClick={() => setActiveTab('console')}
                        className="text-xs text-gray-500 hover:text-white transition-colors"
                      >
                        View Full Console
                      </button>
                    </div>
                    <div className="flex-1 p-4 font-mono text-xs text-gray-400 overflow-y-auto space-y-0.5">
                      {(logs[selectedInstance] || []).map((line, i) => (
                        <div key={i} className={cn(
                          line.includes('ERROR') ? 'text-red-400' :
                            line.includes('WARN') ? 'text-yellow-400' :
                              line.includes('INFO') ? 'text-gray-400' : 'text-gray-300'
                        )}>
                          {line}
                        </div>
                      ))}
                      <div ref={consoleEndRef} />
                      {(!logs[selectedInstance] || logs[selectedInstance].length === 0) && (
                        <div className="text-gray-600 italic">No logs yet. Start the server to see output.</div>
                      )}
                    </div>
                    <form onSubmit={handleSendCommand} className="p-2 bg-[#1a1a1a] border-t border-white/5">
                      <input
                        type="text"
                        value={command}
                        onChange={(e) => setCommand(e.target.value)}
                        placeholder="Type a command..."
                        className="w-full bg-transparent border-none focus:ring-0 text-xs font-mono px-2"
                        autoComplete="off"
                      />
                      <button type="submit" className="hidden" />
                    </form>
                  </div>
                </div>
              )}

              {activeTab === 'console' && (
                <div className="bg-[#000] rounded-lg border border-white/5 flex flex-col h-[calc(100vh-280px)] overflow-hidden">
                  <div className="flex-1 p-4 font-mono text-sm text-gray-400 overflow-y-auto space-y-0.5">
                    {(logs[selectedInstance] || []).map((line, i) => (
                      <div key={i} className={cn(
                        line.includes('ERROR') ? 'text-red-400' :
                          line.includes('WARN') ? 'text-yellow-400' :
                            line.includes('INFO') ? 'text-gray-400' : 'text-gray-300'
                      )}>
                        {line}
                      </div>
                    ))}
                    <div ref={consoleEndRef} />
                    {(!logs[selectedInstance] || logs[selectedInstance].length === 0) && (
                      <div className="text-gray-600 italic">No logs yet. Start the server to see output.</div>
                    )}
                  </div>
                  <form onSubmit={handleSendCommand} className="p-2 bg-[#1a1a1a] border-t border-white/5">
                    <input
                      type="text"
                      value={command}
                      onChange={(e) => setCommand(e.target.value)}
                      placeholder="Type a command..."
                      className="w-full bg-transparent border-none focus:ring-0 text-sm font-mono px-2"
                      autoComplete="off"
                    />
                    <button type="submit" className="hidden" />
                  </form>
                </div>
              )}

              {activeTab !== 'dashboard' && activeTab !== 'console' && (
                <div className="flex flex-col items-center justify-center h-full text-gray-500 py-20">
                  {activeTab === 'plugins' && <Puzzle size={48} className="mb-4 opacity-20" />}
                  {activeTab === 'players' && <Users size={48} className="mb-4 opacity-20" />}
                  {activeTab === 'backups' && <History size={48} className="mb-4 opacity-20" />}
                  {activeTab === 'scheduler' && <Calendar size={48} className="mb-4 opacity-20" />}
                  {activeTab === 'settings' && <Settings size={48} className="mb-4 opacity-20" />}
                  <h3 className="text-lg font-medium text-gray-400 capitalize">{activeTab}</h3>
                  <p>This section is coming soon.</p>
                </div>
              )}
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-gray-500">
            <Database size={64} className="mb-4 opacity-20" />
            <p>Select an instance to manage</p>
          </div>
        )}
      </div>
      {/* Create Instance Modal */}
      <CreateInstanceModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreated={loadInstances}
      />
    </div>
  )
}

export default App
