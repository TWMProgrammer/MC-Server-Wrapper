import { useMemo } from 'react'
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  LineChart,
  Line
} from 'recharts'
import { BarChart3, Cpu, HardDrive, MemoryStick } from 'lucide-react'
import { ResourceUsage } from './types'
import { AppSettings } from './hooks/useAppSettings'

interface StatsTabProps {
  history: ResourceUsage[];
  settings: AppSettings;
}

export function StatsTab({ history, settings }: StatsTabProps) {
  const chartData = useMemo(() => {
    return history.map(h => ({
      ...h,
      time: h.timestamp ? new Date(h.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }) : '',
      cpu: Math.round(h.cpu_usage * 100) / 100,
      memory: Math.round((h.memory_usage / 1024 / 1024) * 100) / 100, // MB
      diskRead: Math.round((h.disk_read / 1024) * 100) / 100, // KB/s
      diskWrite: Math.round((h.disk_write / 1024) * 100) / 100, // KB/s
    }))
  }, [history])

  const latestUsage = history[history.length - 1] || { cpu_usage: 0, memory_usage: 0, disk_read: 0, disk_write: 0 }

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div>
          <h2 className="text-3xl font-black flex items-center gap-3 tracking-tight">
            <BarChart3 className="text-primary" size={32} />
            Statistics
          </h2>
          <p className="text-gray-500 text-sm mt-1 font-medium">
            Real-time resource usage monitoring and historical data.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* CPU Card */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-blue-500/10 text-blue-500 rounded-lg">
              <Cpu size={20} />
            </div>
            <h3 className="font-bold text-lg">CPU Usage</h3>
          </div>
          <div className="text-3xl font-black text-blue-500">
            {Math.round(latestUsage.cpu_usage * 10) / 10}%
          </div>
        </div>

        {/* Memory Card */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-emerald-500/10 text-emerald-500 rounded-lg">
              <MemoryStick size={20} />
            </div>
            <h3 className="font-bold text-lg">Memory</h3>
          </div>
          <div className="text-3xl font-black text-emerald-500">
            {formatBytes(latestUsage.memory_usage)}
          </div>
        </div>

        {/* Disk Card */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-purple-500/10 text-purple-500 rounded-lg">
              <HardDrive size={20} />
            </div>
            <h3 className="font-bold text-lg">Disk I/O</h3>
          </div>
          <div className="flex flex-col">
            <div className="text-lg font-bold text-purple-500">
              Read: {formatBytes(latestUsage.disk_read)}/s
            </div>
            <div className="text-lg font-bold text-purple-400">
              Write: {formatBytes(latestUsage.disk_write)}/s
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6">
        {/* CPU Graph */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <h3 className="font-bold text-lg mb-6 flex items-center gap-2">
            <Cpu size={18} className="text-blue-500" />
            CPU History (%)
          </h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#888888" vertical={false} opacity={0.1} />
                <XAxis 
                  dataKey="time" 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  tickFormatter={(value) => `${value}%`}
                />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: 'rgba(0, 0, 0, 0.8)', 
                    border: 'none', 
                    borderRadius: '8px',
                    color: '#fff'
                  }}
                  itemStyle={{ color: '#3b82f6' }}
                />
                <Area 
                  type="monotone" 
                  dataKey="cpu" 
                  stroke="#3b82f6" 
                  fillOpacity={1} 
                  fill="url(#colorCpu)" 
                  strokeWidth={2}
                  isAnimationActive={false}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Memory Graph */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <h3 className="font-bold text-lg mb-6 flex items-center gap-2">
            <MemoryStick size={18} className="text-emerald-500" />
            Memory History (MB)
          </h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <defs>
                  <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#888888" vertical={false} opacity={0.1} />
                <XAxis 
                  dataKey="time" 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  tickFormatter={(value) => `${value}MB`}
                />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: 'rgba(0, 0, 0, 0.8)', 
                    border: 'none', 
                    borderRadius: '8px',
                    color: '#fff'
                  }}
                  itemStyle={{ color: '#10b981' }}
                />
                <Area 
                  type="monotone" 
                  dataKey="memory" 
                  stroke="#10b981" 
                  fillOpacity={1} 
                  fill="url(#colorMem)" 
                  strokeWidth={2}
                  isAnimationActive={false}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Disk I/O Graph */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <h3 className="font-bold text-lg mb-6 flex items-center gap-2">
            <HardDrive size={18} className="text-purple-500" />
            Disk I/O History (KB/s)
          </h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#888888" vertical={false} opacity={0.1} />
                <XAxis 
                  dataKey="time" 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  interval="preserveStartEnd"
                />
                <YAxis 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false}
                  tickFormatter={(value) => `${value}KB`}
                />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: 'rgba(0, 0, 0, 0.8)', 
                    border: 'none', 
                    borderRadius: '8px',
                    color: '#fff'
                  }}
                />
                <Line 
                  type="monotone" 
                  dataKey="diskRead" 
                  stroke="#a855f7" 
                  strokeWidth={2}
                  dot={false}
                  name="Read"
                  isAnimationActive={false}
                />
                <Line 
                  type="monotone" 
                  dataKey="diskWrite" 
                  stroke="#c084fc" 
                  strokeWidth={2}
                  dot={false}
                  name="Write"
                  isAnimationActive={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  )
}
