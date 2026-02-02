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
    usage,
    history,
    logs,
    loadInstances,
    startServer,
    stopServer,
    sendCommand
  } = useServer()

  const { settings, updateSettings } = useAppSettings()

  const [activeTab, setActiveTab] = useState<TabId>('dashboard')
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showSettingsModal, setShowSettingsModal] = useState(false)
  const [downloadingInstance, setDownloadingInstance] = useState<{ id: string, name: string } | null>(null)
  const [command, setCommand] = useState('')

  const consoleEndRef = useConsoleScroll(logs, selectedInstanceId)

  const handleSendCommand = (e: React.FormEvent) => {
    e.preventDefault()
    if (command) {
      sendCommand(command)
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

  return (
    <div
      className="fixed inset-0 overflow-hidden bg-background"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
      <div className="flex h-full text-gray-900 dark:text-white selection:bg-primary/30">
        <Sidebar
          instances={instances}
          selectedInstanceId={selectedInstanceId}
          onSelectInstance={setSelectedInstanceId}
          onCreateNew={() => setShowCreateModal(true)}
          onOpenSettings={() => setShowSettingsModal(true)}
          onInstancesUpdated={loadInstances}
          settings={settings}
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
                onStartServer={() => startServer()}
                onStopServer={() => stopServer()}
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
