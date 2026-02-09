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
import { ResourceUsage, Instance } from './types'
import { AppSettings } from './hooks/useAppSettings'

interface StatsTabProps {
  history: ResourceUsage[];
  settings: AppSettings;
  currentInstance: Instance;
}

export function StatsTab({ history, settings, currentInstance }: StatsTabProps) {
  const ramUnit = currentInstance.settings.max_ram_unit;
  const isGigabytes = ramUnit === 'G' || ramUnit === 'GB';
  const totalRamBytes = currentInstance.settings.max_ram * (isGigabytes ? 1024 * 1024 * 1024 : 1024 * 1024);

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
            <div className="p-2 bg-emerald-500/10 text-emerald-500 rounded-lg">
              <Cpu size={20} />
            </div>
            <h3 className="font-bold text-lg">CPU Usage</h3>
          </div>
          <div className="text-3xl font-black text-emerald-500">
            {Math.round(latestUsage.cpu_usage * 10) / 10}%
          </div>
        </div>

        {/* Memory Card */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-blue-500/10 text-blue-500 rounded-lg">
              <MemoryStick size={20} />
            </div>
            <h3 className="font-bold text-lg">Memory</h3>
          </div>
          <div className="text-3xl font-black text-blue-500">
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
            <Cpu size={18} className="text-emerald-500" />
            CPU History (%)
          </h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.2} />
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                <XAxis
                  dataKey="time"
                  hide
                />
                <YAxis
                  domain={[0, 100]}
                  tick={{ fill: '#6b7280', fontSize: 10, fontWeight: 600 }}
                  tickFormatter={(value) => `${value}%`}
                  axisLine={false}
                  tickLine={false}
                  width={40}
                />
                <Tooltip
                  cursor={{ stroke: '#10b98120', strokeWidth: 2 }}
                  content={({ active, payload }) => {
                    if (active && payload && payload.length) {
                      return (
                        <div className="bg-surface/90 backdrop-blur-md border border-white/10 p-2 px-3 rounded-lg shadow-2xl">
                          <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                            <div className="flex flex-col">
                              <p className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">{payload[0].payload.time}</p>
                              <p className="text-sm font-bold text-white">
                                {Number(payload[0].value).toFixed(1)}%
                              </p>
                            </div>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="cpu"
                  stroke="#10b981"
                  fillOpacity={1}
                  fill="url(#colorCpu)"
                  strokeWidth={2}
                  isAnimationActive={false}
                  activeDot={{ r: 4, fill: '#10b981', strokeWidth: 0 }}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Memory Graph */}
        <div className="bg-surface/50 border border-black/5 dark:border-white/5 rounded-2xl p-6">
          <h3 className="font-bold text-lg mb-6 flex items-center gap-2">
            <MemoryStick size={18} className="text-blue-500" />
            Memory History (MB)
          </h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.2} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                <XAxis
                  dataKey="time"
                  hide
                />
                <YAxis
                  domain={[0, totalRamBytes / 1024 / 1024]}
                  tick={{ fill: '#6b7280', fontSize: 10, fontWeight: 600 }}
                  tickFormatter={(value) => `${Math.round(value)}MB`}
                  axisLine={false}
                  tickLine={false}
                  width={50}
                />
                <Tooltip
                  cursor={{ stroke: '#3b82f620', strokeWidth: 2 }}
                  content={({ active, payload }) => {
                    if (active && payload && payload.length) {
                      return (
                        <div className="bg-surface/90 backdrop-blur-md border border-white/10 p-2 px-3 rounded-lg shadow-2xl">
                          <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-blue-500" />
                            <div className="flex flex-col">
                              <p className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">{payload[0].payload.time}</p>
                              <p className="text-sm font-bold text-white">
                                {formatBytes(Number(payload[0].value) * 1024 * 1024)}
                              </p>
                            </div>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="memory"
                  stroke="#3b82f6"
                  fillOpacity={1}
                  fill="url(#colorMem)"
                  strokeWidth={2}
                  isAnimationActive={false}
                  activeDot={{ r: 4, fill: '#3b82f6', strokeWidth: 0 }}
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
              <LineChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                <XAxis
                  dataKey="time"
                  hide
                />
                <YAxis
                  tick={{ fill: '#6b7280', fontSize: 10, fontWeight: 600 }}
                  tickFormatter={(value) => `${value}KB`}
                  axisLine={false}
                  tickLine={false}
                  width={50}
                />
                <Tooltip
                  cursor={{ stroke: '#a855f720', strokeWidth: 2 }}
                  content={({ active, payload }) => {
                    if (active && payload && payload.length) {
                      return (
                        <div className="bg-surface/90 backdrop-blur-md border border-white/10 p-2 px-3 rounded-lg shadow-2xl">
                          <p className="text-[10px] font-bold text-gray-400 uppercase tracking-widest mb-2">{payload[0].payload.time}</p>
                          <div className="space-y-1.5">
                            {payload.map((entry, index) => (
                              <div key={index} className="flex items-center gap-2">
                                <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: entry.color }} />
                                <p className="text-sm font-bold text-white">
                                  {entry.name}: {formatBytes(Number(entry.value) * 1024)}/s
                                </p>
                              </div>
                            ))}
                          </div>
                        </div>
                      );
                    }
                    return null;
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
                  activeDot={{ r: 4, fill: '#a855f7', strokeWidth: 0 }}
                />
                <Line
                  type="monotone"
                  dataKey="diskWrite"
                  stroke="#c084fc"
                  strokeWidth={2}
                  dot={false}
                  name="Write"
                  isAnimationActive={false}
                  activeDot={{ r: 4, fill: '#c084fc', strokeWidth: 0 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  )
}
