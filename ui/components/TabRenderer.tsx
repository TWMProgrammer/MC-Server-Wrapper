import { motion } from 'framer-motion'
import {
  Settings,
  Puzzle,
  Layers,
  History,
  Calendar
} from 'lucide-react'
import { Dashboard } from './Dashboard'
import { Console } from './Console'
import { LogsTab } from '../LogsTab'
import { PlayersTab } from '../PlayersTab'
import { ConfigTab } from '../ConfigTab'
import { BackupsTab } from '../BackupsTab'
import { SchedulesTab } from '../SchedulesTab'
import { TabId, Instance, ResourceUsage } from '../types'
import { AppSettings } from '../hooks/useAppSettings'

import { InstanceSettingsTab } from '../InstanceSettingsTab'

interface TabRendererProps {
  activeTab: TabId;
  selectedInstanceId: string;
  currentInstance: Instance;
  usage: ResourceUsage | null;
  history: any[];
  logs: string[];
  consoleEndRef: React.RefObject<HTMLDivElement | null>;
  command: string;
  onCommandChange: (val: string) => void;
  onSendCommand: (e: React.FormEvent) => void;
  onSetActiveTab: (tab: TabId) => void;
  onInstancesUpdated?: () => void;
  settings: AppSettings;
}

export function TabRenderer({
  activeTab,
  selectedInstanceId,
  currentInstance,
  usage,
  history,
  logs,
  consoleEndRef,
  command,
  onCommandChange,
  onSendCommand,
  onSetActiveTab,
  onInstancesUpdated,
  settings
}: TabRendererProps) {
  if (activeTab === 'dashboard') {
    return (
      <div className="space-y-6">
        <Dashboard
          currentInstance={currentInstance}
          usage={usage}
          history={history}
          settings={settings}
        />

        {/* Dashboard Console Preview */}
        <div className="mt-6">
          <Console
            logs={logs}
            consoleEndRef={consoleEndRef}
            command={command}
            onCommandChange={onCommandChange}
            onSendCommand={onSendCommand}
            onViewFull={() => onSetActiveTab('console')}
          />
        </div>
      </div>
    );
  }

  if (activeTab === 'console') {
    return (
      <Console
        isFull
        logs={logs}
        consoleEndRef={consoleEndRef}
        command={command}
        onCommandChange={onCommandChange}
        onSendCommand={onSendCommand}
      />
    );
  }

  if (activeTab === 'logs') {
    return <LogsTab instanceId={selectedInstanceId} />;
  }

  if (activeTab === 'players') {
    return <PlayersTab instanceId={selectedInstanceId} settings={settings} />;
  }

  if (activeTab === 'config') {
    return <ConfigTab instanceId={selectedInstanceId} />;
  }

  if (activeTab === 'backups') {
    return <BackupsTab instanceId={selectedInstanceId} />;
  }

  if (activeTab === 'settings') {
    return <InstanceSettingsTab instance={currentInstance} onUpdate={onInstancesUpdated} />;
  }

  if (activeTab === 'scheduler') {
    return <SchedulesTab instanceId={selectedInstanceId} />;
  }

  // Placeholder for other tabs
  return (
    <div className="flex flex-col items-center justify-center h-full text-gray-400 py-20 bg-surface/50 rounded-2xl border border-black/5 dark:border-white/5">
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.1 }}
      >
        {activeTab === 'plugins' && <Puzzle size={64} className="mb-4 text-primary opacity-40" />}
        {activeTab === 'mods' && <Layers size={64} className="mb-4 text-primary opacity-40" />}
      </motion.div>
      <h3 className="text-2xl font-semibold capitalize mb-2">{activeTab}</h3>
      <p className="text-gray-500">This feature is currently under development.</p>
    </div>
  );
}
