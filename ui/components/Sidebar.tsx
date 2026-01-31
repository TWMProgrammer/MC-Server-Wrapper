import { Database, Plus } from 'lucide-react'
import { Instance } from '../types'
import { cn } from '../utils'

interface SidebarProps {
  instances: Instance[];
  selectedInstanceId: string | null;
  onSelectInstance: (id: string) => void;
  onCreateNew: () => void;
}

export function Sidebar({ instances, selectedInstanceId, onSelectInstance, onCreateNew }: SidebarProps) {
  return (
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
            onClick={() => onSelectInstance(inst.id)}
            className={cn(
              "w-full text-left px-3 py-2 rounded transition-colors flex items-center gap-2",
              selectedInstanceId === inst.id ? "bg-blue-600 text-white shadow-lg" : "hover:bg-white/5 text-gray-300"
            )}
          >
            <div className={cn(
              "w-2 h-2 rounded-full",
              inst.status === 'Running' ? "bg-green-500" :
                inst.status === 'Starting' ? "bg-orange-500" :
                  (inst.status === 'Stopping' || inst.status === 'Crashed') ? "bg-red-500" : "bg-gray-500"
            )} />
            <span className="truncate">{inst.name}</span>
          </button>
        ))}
        <button
          onClick={onCreateNew}
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
  )
}
