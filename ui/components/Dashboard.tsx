import { Activity } from 'lucide-react'
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'
import { Instance, ResourceUsage } from '../types'

interface DashboardProps {
  currentInstance: Instance;
  usage: ResourceUsage | null;
  history: ResourceUsage[];
}

export function Dashboard({
  currentInstance,
  usage,
  history
}: DashboardProps) {
  return (
    <div className="space-y-6">
      {/* Dashboard Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
          <div className="text-gray-400 text-sm mb-1">CPU Usage</div>
          <div className="text-2xl font-mono">{usage?.cpu_usage.toFixed(1) || '0.0'}%</div>
        </div>
        <div className="bg-[#242424] p-4 rounded-lg border border-white/5">
          <div className="text-gray-400 text-sm mb-1">Memory Usage</div>
          <div className="text-2xl font-mono">
            {(usage?.memory_usage || 0) / 1024 / 1024 > 1024 
              ? `${((usage?.memory_usage || 0) / 1024 / 1024 / 1024).toFixed(2)} GB` 
              : `${((usage?.memory_usage || 0) / 1024 / 1024).toFixed(0)} MB`}
          </div>
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
    </div>
  )
}
