import { useState, useEffect, useRef } from 'react'
import {
  Settings,
  LayoutDashboard,
  Database,
  Users,
  Terminal,
  Puzzle,
  Layers,
  History,
  Calendar,
  FileText
} from 'lucide-react'
import { CreateInstanceModal } from './CreateInstanceModal'
import { DownloadProgressModal } from './DownloadProgressModal'
import { LogsTab } from './LogsTab'
import { useServer } from './hooks/useServer'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { Dashboard } from './components/Dashboard'
import { Console } from './components/Console'
import { TabId } from './types'

function App() {
  const {
    instances,
    selectedInstanceId,
    setSelectedInstanceId,
    currentInstance,
    status,
    usage,
    history,
    logs,
    loadInstances,
    startServer,
    stopServer,
    sendCommand
  } = useServer()

  const [activeTab, setActiveTab] = useState<TabId>('dashboard')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [downloadingInstance, setDownloadingInstance] = useState<{ id: string, name: string } | null>(null)
  const [command, setCommand] = useState('')
  const consoleEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [logs, selectedInstanceId])

  const handleSendCommand = (e: React.FormEvent) => {
    e.preventDefault()
    if (command) {
      sendCommand(command)
      setCommand('')
    }
  }

  const supportsPlugins = (loader?: string) => {
    if (!loader) return false;
    const l = loader.toLowerCase();
    return ['paper', 'purpur', 'spigot', 'bukkit'].includes(l);
  };

  const supportsMods = (loader?: string) => {
    if (!loader) return false;
    const l = loader.toLowerCase();
    return ['fabric', 'forge', 'neoforge', 'quilt'].includes(l);
  };

  const ALL_TABS: { id: TabId; label: string; icon: any }[] = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'console', label: 'Console', icon: Terminal },
    { id: 'logs', label: 'Logs', icon: FileText },
    { id: 'plugins', label: 'Plugins', icon: Puzzle },
    { id: 'mods', label: 'Mods', icon: Layers },
    { id: 'players', label: 'Players', icon: Users },
    { id: 'backups', label: 'Backups', icon: History },
    { id: 'scheduler', label: 'Scheduler', icon: Calendar },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  const tabs = ALL_TABS.filter(tab => {
    if (tab.id === 'plugins') return supportsPlugins(currentInstance?.mod_loader);
    if (tab.id === 'mods') return supportsMods(currentInstance?.mod_loader);
    return true;
  });

  // Ensure active tab is valid for current instance
  useEffect(() => {
    if (currentInstance) {
      const isValid = tabs.some(t => t.id === activeTab);
      if (!isValid) {
        setActiveTab('dashboard');
      }
    }
  }, [currentInstance, activeTab, tabs]);

  return (
    <div className="flex h-screen bg-[#1a1a1a] text-white">
      <Sidebar
        instances={instances}
        selectedInstanceId={selectedInstanceId}
        onSelectInstance={setSelectedInstanceId}
        onCreateNew={() => setShowCreateModal(true)}
      />

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden bg-[#1e1e1e]">
        {selectedInstanceId && currentInstance ? (
          <>
            <Header
              currentInstance={currentInstance}
              status={status}
              activeTab={activeTab}
              tabs={tabs}
              onStartServer={startServer}
              onStopServer={stopServer}
              onSetActiveTab={setActiveTab}
              onInstancesUpdated={loadInstances}
            />

            <div className="flex-1 overflow-y-auto p-6">
              {activeTab === 'dashboard' && (
                <div className="space-y-6">
                  <Dashboard
                    currentInstance={currentInstance}
                    usage={usage}
                    history={history}
                  />

                  {/* Dashboard Console Preview */}
                  <Console
                    logs={logs[selectedInstanceId] || []}
                    consoleEndRef={consoleEndRef}
                    command={command}
                    onCommandChange={setCommand}
                    onSendCommand={handleSendCommand}
                    onViewFull={() => setActiveTab('console')}
                  />
                </div>
              )}

              {activeTab === 'console' && (
                <Console
                  isFull
                  logs={logs[selectedInstanceId] || []}
                  consoleEndRef={consoleEndRef}
                  command={command}
                  onCommandChange={setCommand}
                  onSendCommand={handleSendCommand}
                />
              )}

              {activeTab === 'logs' && (
                <LogsTab instanceId={selectedInstanceId} />
              )}

              {activeTab !== 'dashboard' && activeTab !== 'console' && activeTab !== 'logs' && (
                <div className="flex flex-col items-center justify-center h-full text-gray-500 py-20">
                  {activeTab === 'plugins' && <Puzzle size={48} className="mb-4 opacity-20" />}
                  {activeTab === 'mods' && <Layers size={48} className="mb-4 opacity-20" />}
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

      <CreateInstanceModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreated={(instance) => {
          loadInstances(instance.id)
          setActiveTab('dashboard')
          setShowCreateModal(false)
        }}
      />

      <DownloadProgressModal
        isOpen={!!downloadingInstance}
        onClose={() => setDownloadingInstance(null)}
        instanceId={downloadingInstance?.id || null}
        instanceName={downloadingInstance?.name || ''}
      />
    </div>
  )
}

export default App
