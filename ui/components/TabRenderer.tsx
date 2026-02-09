import {
  Settings,
  Puzzle,
  Layers,
  History,
  Calendar,
  Box
} from 'lucide-react'
import { Dashboard } from './Dashboard'
import { Console } from './Console'
import { LogsTab } from '../LogsTab'
import { PlayersTab } from '../PlayersTab'
import { ConfigTab } from '../ConfigTab'
import { BackupsTab } from '../BackupsTab'
import { SchedulesTab } from '../SchedulesTab'
import { PluginsTab } from '../PluginsTab'
import { ModsTab } from '../ModsTab'
import { StatsTab } from '../StatsTab'
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
  commandHistory?: string[];
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
  commandHistory,
  onCommandChange,
  onSendCommand,
  onSetActiveTab,
  onInstancesUpdated,
  settings
}: TabRendererProps) {
  const renderTabContent = () => {
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
              commandHistory={commandHistory}
              onCommandChange={onCommandChange}
              onSendCommand={onSendCommand}
              onViewFull={() => onSetActiveTab('console')}
              settings={settings}
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
          commandHistory={commandHistory}
          onCommandChange={onCommandChange}
          onSendCommand={onSendCommand}
          settings={settings}
        />
      );
    }

    if (activeTab === 'logs') {
      return <LogsTab instanceId={selectedInstanceId} />;
    }

    if (activeTab === 'stats') {
      return <StatsTab history={history} settings={settings} currentInstance={currentInstance} />;
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

    if (activeTab === 'plugins') {
      return <PluginsTab instanceId={selectedInstanceId} />;
    }

    if (activeTab === 'mods') {
      return <ModsTab instanceId={selectedInstanceId} />;
    }

    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 py-20 bg-surface/50 rounded-2xl border border-black/5 dark:border-white/5">
        <div className="w-24 h-24 rounded-full bg-black/5 dark:bg-white/[0.02] border border-black/5 dark:border-white/5 flex items-center justify-center mb-6">
          <Box size={48} strokeWidth={1} />
        </div>
        <h3 className="text-2xl font-black text-gray-400 dark:text-white/30 uppercase tracking-[0.2em] mb-2">{activeTab}</h3>
        <p className="text-sm font-medium leading-relaxed text-gray-500 dark:text-white/40">This feature is currently under development.</p>
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {renderTabContent()}
    </div>
  );
}
