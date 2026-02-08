import { useState, useEffect } from 'react'
import { AnimatePresence, motion } from 'framer-motion'
import { CreateInstanceModal } from './CreateInstanceModal'
import { DownloadProgressModal } from './DownloadProgressModal'
import { AppSettingsModal } from './components/AppSettingsModal'
import { useServer } from './hooks/useServer'
import { useAppSettings } from './hooks/useAppSettings'
import { useConsoleScroll } from './hooks/useConsoleScroll'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { TitleBar } from './components/TitleBar'
import { EmptyState } from './components/EmptyState'
import { GlobalDashboard } from './components/GlobalDashboard'
import { TabRenderer } from './components/TabRenderer'
import { getAvailableTabs } from './utils/tabUtils'
import { TabId } from './types'
import { cn } from './utils'

function App() {
  const {
    instances,
    selectedInstanceId,
    setSelectedInstanceId,
    currentInstance,
    status,
    isTransitioning,
    usage,
    history,
    logs,
    loadInstances,
    startServer,
    stopServer,
    restartServer,
    sendCommand,
    loading: serverLoading
  } = useServer()

  const { settings, updateSettings, isLoading: settingsLoading } = useAppSettings()

  const [activeTab, setActiveTab] = useState<TabId>('dashboard')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showSettingsModal, setShowSettingsModal] = useState(false)
  const [downloadingInstance, setDownloadingInstance] = useState<{ id: string, name: string } | null>(null)
  const [command, setCommand] = useState('')
  const [commandHistory, setCommandHistory] = useState<string[]>([])

  const consoleEndRef = useConsoleScroll(logs, selectedInstanceId)

  const handleSendCommand = (e: React.FormEvent) => {
    e.preventDefault()
    if (command.trim()) {
      sendCommand(command)
      setCommandHistory(prev => {
        // Don't add if it's the same as the last command
        if (prev.length > 0 && prev[prev.length - 1] === command) {
          return prev;
        }
        return [...prev, command].slice(-50); // Keep last 50 commands
      })
      setCommand('')
    }
  }

  const tabs = getAvailableTabs(currentInstance?.mod_loader);

  // Ensure active tab is valid for current instance
  useEffect(() => {
    if (currentInstance) {
      const isValid = tabs.some(t => t.id === activeTab);
      if (!isValid) {
        setActiveTab('dashboard');
      }
    }
  }, [currentInstance, activeTab, tabs]);

  const isAppLoading = serverLoading || settingsLoading;

  if (isAppLoading) {
    return (
      <div className="flex flex-col items-center justify-center h-screen w-screen bg-[#0a0a0c] text-white font-sans">
        <div className="mb-6">
          <div className="w-16 h-16 rounded-full border-4 border-primary/10 border-t-primary animate-spin" />
        </div>
        <div className="text-sm font-medium tracking-widest uppercase opacity-80 animate-pulse">
          {settingsLoading ? "Loading Settings..." : "Initializing Instances..."}
        </div>
      </div>
    );
  }

  return (
    <div
      className="fixed inset-0 overflow-hidden bg-background flex flex-col"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
      <TitleBar />
      <div className="flex flex-1 overflow-hidden pt-10 text-gray-900 dark:text-white selection:bg-primary/30">
        <Sidebar
          instances={instances}
          selectedInstanceId={selectedInstanceId}
          onSelectInstance={setSelectedInstanceId}
          onCreateNew={() => setShowCreateModal(true)}
          onOpenSettings={() => setShowSettingsModal(true)}
          onInstancesUpdated={loadInstances}
        />

        {/* Main Content */}
        <div className="flex-1 flex flex-col overflow-hidden bg-surface/30 backdrop-blur-sm">
          {selectedInstanceId && currentInstance ? (
            <>
              <Header
                currentInstance={currentInstance}
                status={status}
                isTransitioning={selectedInstanceId ? isTransitioning[selectedInstanceId] : null}
                activeTab={activeTab}
                usage={usage}
                tabs={tabs}
                onStartServer={() => startServer()}
                onStopServer={() => stopServer()}
                onRestartServer={() => restartServer()}
                onSetActiveTab={setActiveTab}
                onInstancesUpdated={loadInstances}
              />

              <div className={cn(
                "flex-1 min-h-0",
                activeTab === 'console' ? "overflow-hidden" : "overflow-y-auto custom-scrollbar"
              )}>
                <AnimatePresence mode="wait">
                  <motion.div
                    key={activeTab}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -10 }}
                    transition={{ duration: 0.2, ease: "easeOut" }}
                    className={cn(
                      "p-6",
                      activeTab === 'console' ? "h-full" : "min-h-full"
                    )}
                  >
                    <TabRenderer
                      activeTab={activeTab}
                      selectedInstanceId={selectedInstanceId}
                      currentInstance={currentInstance}
                      usage={usage}
                      history={history}
                      logs={logs[selectedInstanceId] || []}
                      consoleEndRef={consoleEndRef}
                      command={command}
                      onCommandChange={setCommand}
                      onSendCommand={handleSendCommand}
                      commandHistory={commandHistory}
                      onSetActiveTab={setActiveTab}
                      onInstancesUpdated={loadInstances}
                      settings={settings}
                    />
                  </motion.div>
                </AnimatePresence>
              </div>
            </>
          ) : (
            instances.length === 0 ? (
              <EmptyState />
            ) : (
              <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                <GlobalDashboard
                  instances={instances}
                  isTransitioning={isTransitioning}
                  onSelectInstance={setSelectedInstanceId}
                  onStartServer={startServer}
                  onStopServer={stopServer}
                  settings={settings}
                />
              </div>
            )
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
          settings={settings}
          updateSettings={updateSettings}
        />
      </div>
    </div>
  )
}

export default App
