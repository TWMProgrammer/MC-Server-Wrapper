import { Database, Network, Beaker, Users, Tag, Play, Square } from 'lucide-react'
import { Instance, TabId } from '../types'
import { cn } from '../utils'
import { InstanceFolderDropdown } from '../InstanceFolderDropdown'
import { InstanceSettingsDropdown } from '../InstanceSettingsDropdown'

interface HeaderProps {
  currentInstance: Instance;
  status: string;
  activeTab: TabId;
  tabs: { id: TabId; label: string; icon: any }[];
  onStartServer: () => void;
  onStopServer: () => void;
  onSetActiveTab: (tab: TabId) => void;
  onInstancesUpdated: () => void;
}

export function Header({
  currentInstance,
  status,
  activeTab,
  tabs,
  onStartServer,
  onStopServer,
  onSetActiveTab,
  onInstancesUpdated
}: HeaderProps) {
  return (
    <div className="px-6 pt-6 pb-2 bg-[#242424]">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 bg-blue-600 rounded-lg flex items-center justify-center shadow-lg">
            <Database size={24} />
          </div>
          <div>
            <div className="flex items-center gap-3">
              <h2 className="text-2xl font-bold">{currentInstance.name}</h2>
              <div className="flex items-center gap-1.5">
                <div className={cn(
                  "w-2 h-2 rounded-full",
                  status === 'Running' ? "bg-green-500" :
                    status === 'Starting' ? "bg-orange-500" :
                      (status === 'Stopping' || status === 'Crashed') ? "bg-red-500" : "bg-gray-500"
                )} />
                <span className={cn(
                  "text-sm font-medium",
                  status === 'Running' ? "text-green-400" :
                    status === 'Starting' ? "text-orange-400" :
                      (status === 'Stopping' || status === 'Crashed') ? "text-red-400" : "text-gray-400"
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
              onClick={onStartServer}
              className="flex items-center gap-2 px-6 py-2 bg-[#2d333b] hover:bg-[#343b44] text-green-500 rounded-md font-bold transition-colors border border-white/5"
            >
              <Play size={18} fill="currentColor" />
              Start
            </button>
          ) : (
            <button
              onClick={onStopServer}
              className="flex items-center gap-2 px-6 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md font-bold transition-colors shadow-lg"
            >
              <Square size={18} fill="currentColor" />
              Stop
            </button>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between border-t border-white/5 mt-4">
        <div className="flex gap-1">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => onSetActiveTab(tab.id)}
              className={cn(
                "px-4 py-3 text-sm font-medium transition-all relative",
                activeTab === tab.id ? "text-white" : "text-gray-500 hover:text-gray-300"
              )}
            >
              <div className="flex items-center gap-2">
                <tab.icon size={14} className={activeTab === tab.id ? "text-blue-400" : "text-gray-500"} />
                {tab.label}
              </div>
              {activeTab === tab.id && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500" />
              )}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-4 text-gray-500">
          <InstanceFolderDropdown
            instance={{
              id: currentInstance.id,
              name: currentInstance.name
            }}
          />
          <InstanceSettingsDropdown
            instance={{
              id: currentInstance.id,
              name: currentInstance.name
            }}
            onUpdated={onInstancesUpdated}
          />
        </div>
      </div>
    </div>
  )
}
