import { useState, useEffect, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
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
  FileText,
  Sliders
} from 'lucide-react'
import { CreateInstanceModal } from './CreateInstanceModal'
import { DownloadProgressModal } from './DownloadProgressModal'
import { AppSettingsModal } from './components/AppSettingsModal'
import { LogsTab } from './LogsTab'
import { ConfigTab } from './ConfigTab'
import { useServer } from './hooks/useServer'
import { useAppSettings } from './hooks/useAppSettings'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { Dashboard } from './components/Dashboard'
import { Console } from './components/Console'
import { PlayersTab } from './PlayersTab'
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

  const { accentColor, setAccentColor, theme, setTheme } = useAppSettings()

  const [activeTab, setActiveTab] = useState<TabId>('dashboard')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showSettingsModal, setShowSettingsModal] = useState(false)
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
    { id: 'players', label: 'Players', icon: Users },
    { id: 'config', label: 'Config', icon: Sliders },
    { id: 'plugins', label: 'Plugins', icon: Puzzle },
    { id: 'mods', label: 'Mods', icon: Layers },
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
    <div className="flex h-screen bg-background text-gray-900 dark:text-white selection:bg-primary/30">
      <Sidebar
        instances={instances}
        selectedInstanceId={selectedInstanceId}
        onSelectInstance={setSelectedInstanceId}
        onCreateNew={() => setShowCreateModal(true)}
        onOpenSettings={() => setShowSettingsModal(true)}
      />

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden bg-surface/30 backdrop-blur-sm">
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

            <div className="flex-1 overflow-y-auto custom-scrollbar">
              <AnimatePresence mode="wait">
                <motion.div
                  key={activeTab}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2, ease: "easeOut" }}
                  className="p-6 h-full"
                >
                  {activeTab === 'dashboard' && (
                    <div className="space-y-6">
                      <Dashboard
                        currentInstance={currentInstance}
                        usage={usage}
                        history={history}
                      />

                      {/* Dashboard Console Preview */}
                      <div className="mt-6">
                        <Console
                          logs={logs[selectedInstanceId] || []}
                          consoleEndRef={consoleEndRef}
                          command={command}
                          onCommandChange={setCommand}
                          onSendCommand={handleSendCommand}
                          onViewFull={() => setActiveTab('console')}
                        />
                      </div>
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

                  {activeTab === 'players' && (
                    <PlayersTab instanceId={selectedInstanceId} />
                  )}

                  {activeTab === 'config' && (
                    <ConfigTab instanceId={selectedInstanceId} />
                  )}

                  {activeTab !== 'dashboard' && activeTab !== 'console' && activeTab !== 'logs' && activeTab !== 'players' && activeTab !== 'config' && (
                    <div className="flex flex-col items-center justify-center h-full text-gray-400 py-20 bg-surface/50 rounded-2xl border border-black/5 dark:border-white/5">
                      <motion.div
                        initial={{ scale: 0.8, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        transition={{ delay: 0.1 }}
                      >
                        {activeTab === 'plugins' && <Puzzle size={64} className="mb-4 text-primary opacity-40" />}
                        {activeTab === 'mods' && <Layers size={64} className="mb-4 text-primary opacity-40" />}
                        {activeTab === 'backups' && <History size={64} className="mb-4 text-primary opacity-40" />}
                        {activeTab === 'scheduler' && <Calendar size={64} className="mb-4 text-primary opacity-40" />}
                        {activeTab === 'settings' && <Settings size={64} className="mb-4 text-primary opacity-40" />}
                      </motion.div>
                      <h3 className="text-2xl font-semibold capitalize mb-2">{activeTab}</h3>
                      <p className="text-gray-500">This feature is currently under development.</p>
                    </div>
                  )}
                </motion.div>
              </AnimatePresence>
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center relative overflow-hidden">
            {/* Background decorative elements */}
            <div className="absolute top-1/4 -left-20 w-96 h-96 bg-primary/10 rounded-full blur-[100px] -z-10" />
            <div className="absolute bottom-1/4 -right-20 w-96 h-96 bg-accent-rose/5 rounded-full blur-[100px] -z-10" />

            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
              className="flex flex-col items-center text-center px-6"
            >
              <div className="relative mb-10">
                <motion.div
                  animate={{
                    scale: [1, 1.1, 1],
                    rotate: [0, 5, -5, 0]
                  }}
                  transition={{
                    duration: 6,
                    repeat: Infinity,
                    ease: "easeInOut"
                  }}
                  className="p-10 rounded-[3rem] bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 shadow-2xl relative z-10 backdrop-blur-sm"
                >
                  <Database size={80} className="text-primary" />
                </motion.div>
                {/* Glow effect */}
                <div className="absolute inset-0 bg-primary/20 blur-[50px] -z-10 rounded-full scale-75" />
              </div>

              <h2 className="text-4xl font-black text-gray-900 dark:text-white mb-4 tracking-tighter">
                Ready to <span className="text-transparent bg-clip-text bg-gradient-to-r from-primary to-accent-rose">Craft?</span>
              </h2>
              <p className="text-lg text-gray-500 dark:text-white/40 max-w-md leading-relaxed font-medium">
                Select a server instance from the sidebar to manage your world, or create a brand new one to start your adventure.
              </p>

              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.5 }}
                className="mt-12 flex flex-col items-center gap-4"
              >
                <div className="flex items-center gap-3 px-6 py-3 rounded-2xl bg-black/5 dark:bg-white/[0.03] border border-black/5 dark:border-white/5 text-xs font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">
                  <div className="w-1.5 h-1.5 rounded-full bg-primary shadow-glow-primary animate-pulse" />
                  System Online & Ready
                </div>
              </motion.div>
            </motion.div>
          </div>
        )}
      </div>

      <AnimatePresence>
        {showCreateModal && (
          <CreateInstanceModal
            isOpen={showCreateModal}
            onClose={() => setShowCreateModal(false)}
            onCreated={(instance) => {
              loadInstances(instance.id)
              setActiveTab('dashboard')
              setShowCreateModal(false)
            }}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {downloadingInstance && (
          <DownloadProgressModal
            isOpen={!!downloadingInstance}
            onClose={() => setDownloadingInstance(null)}
            instanceId={downloadingInstance?.id || null}
            instanceName={downloadingInstance?.name || ''}
          />
        )}
      </AnimatePresence>

      <AppSettingsModal
        isOpen={showSettingsModal}
        onClose={() => setShowSettingsModal(false)}
        accentColor={accentColor}
        onAccentColorChange={setAccentColor}
        theme={theme}
        onThemeChange={setTheme}
      />
    </div>
  )
}

export default App
