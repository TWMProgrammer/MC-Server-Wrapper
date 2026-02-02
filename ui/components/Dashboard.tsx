import { Activity, Cpu, HardDrive, Users, Clock } from 'lucide-react'
import { motion } from 'framer-motion'
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'
import { Instance, ResourceUsage } from '../types'
import { AppSettings } from '../hooks/useAppSettings'

interface DashboardProps {
  currentInstance: Instance;
  usage: ResourceUsage | null;
  history: ResourceUsage[];
  settings: AppSettings;
}

export function Dashboard({
  currentInstance,
  usage,
  history,
  settings
}: DashboardProps) {
  const totalRamBytes = currentInstance.settings.ram * (currentInstance.settings.ram_unit === 'GB' ? 1024 * 1024 * 1024 : 1024 * 1024);

  const formatMemory = (bytes: number) => {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024 / 1024).toFixed(1)}GB`;
    return `${(bytes / 1024 / 1024).toFixed(0)}MB`;
  };

  const stats = [
    {
      label: 'CPU Usage',
      value: `${usage?.cpu_usage.toFixed(1) || '0.0'}%`,
      icon: Cpu,
      color: 'text-accent-emerald',
      bg: 'bg-accent-emerald/10',
    },
    {
      label: 'Memory Usage',
      value: (usage?.memory_usage || 0) / 1024 / 1024 > 1024
        ? `${((usage?.memory_usage || 0) / 1024 / 1024 / 1024).toFixed(2)} GB`
        : `${((usage?.memory_usage || 0) / 1024 / 1024).toFixed(0)} MB`,
      icon: HardDrive,
      color: 'text-primary',
      bg: 'bg-primary/10',
    },
    {
      label: 'Players Online',
      value: `0 / ${currentInstance.max_players}`,
      icon: Users,
      color: 'text-accent-amber',
      bg: 'bg-accent-amber/10',
    },
    {
      label: 'Server Uptime',
      value: '00:00:00',
      icon: Clock,
      color: 'text-accent-indigo',
      bg: 'bg-accent-indigo/10',
    },
  ];

  return (
    <div className="space-y-8 animate-fade-in">
      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat, i) => (
          <motion.div
            key={stat.label}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
            className="card group hover:scale-[1.02] transition-all duration-300"
          >
            <div className="flex items-start justify-between">
              <div>
                <p className="text-gray-500 text-xs font-bold uppercase tracking-widest mb-1 transition-colors duration-300">{stat.label}</p>
                <h3 className="text-2xl font-black font-mono tracking-tight text-gray-900 dark:text-white transition-colors duration-300">{stat.value}</h3>
              </div>
              <div className={`p-3 rounded-xl ${stat.bg} ${stat.color} transition-all duration-300 group-hover:rotate-12`}>
                <stat.icon size={20} />
              </div>
            </div>
            <div className="mt-4 flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-black/5 dark:bg-white/10 rounded-full overflow-hidden transition-colors duration-300">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{
                    width: stat.label === 'CPU Usage'
                      ? `${usage?.cpu_usage || 0}%`
                      : stat.label === 'Memory Usage'
                        ? `${((usage?.memory_usage || 0) / totalRamBytes) * 100}%`
                        : '0%'
                  }}
                  className={`h-full ${stat.color.replace('text-', 'bg-')} opacity-80 shadow-[0_0_10px_rgba(0,0,0,0.2)]`}
                />
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Charts Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.4 }}
          className="card bg-white dark:bg-surface border border-black/10 dark:border-white/5"
        >
          <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-accent-emerald/10 rounded-lg text-accent-emerald transition-colors duration-300">
                <Activity size={18} />
              </div>
              <h3 className="font-bold text-lg text-gray-900 dark:text-white transition-colors duration-300">Processor Load</h3>
            </div>
            <div className="text-[10px] font-bold text-gray-500 uppercase tracking-widest transition-colors duration-300">Live Updates</div>
          </div>
          <div className="h-64 w-full pr-4">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={history} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.2} />
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                <XAxis
                  dataKey="timestamp"
                  hide
                />
                <YAxis
                  domain={[0, 100]}
                  tick={{ fill: '#6b7280', fontSize: 10, fontWeight: 600 }}
                  tickFormatter={(val) => `${val}%`}
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
                            <div className="w-1.5 h-1.5 rounded-full bg-accent-emerald" />
                            <p className="text-sm font-bold text-white">
                              {Number(payload[0].value).toFixed(1)}%
                            </p>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="cpu_usage"
                  stroke="#10b981"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#colorCpu)"
                  isAnimationActive={false}
                  activeDot={{ r: 4, fill: '#10b981', strokeWidth: 0 }}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.5 }}
          className="card bg-white dark:bg-surface border border-black/10 dark:border-white/5"
        >
          <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-primary/10 rounded-lg text-primary">
                <Activity size={18} />
              </div>
              <h3 className="font-bold text-lg text-gray-900 dark:text-white transition-colors duration-300">Memory Allocation</h3>
            </div>
            <div className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">Real-time</div>
          </div>
          <div className="h-64 w-full pr-4">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={history} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.2} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                <XAxis
                  dataKey="timestamp"
                  hide
                />
                <YAxis
                  domain={[0, totalRamBytes]}
                  tick={{ fill: '#6b7280', fontSize: 10, fontWeight: 600 }}
                  tickFormatter={(val) => formatMemory(val)}
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
                            <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                            <p className="text-sm font-bold text-white">
                              {formatMemory(Number(payload[0].value))}
                            </p>
                          </div>
                        </div>
                      );
                    }
                    return null;
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="memory_usage"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#colorMem)"
                  isAnimationActive={false}
                  activeDot={{ r: 4, fill: '#3b82f6', strokeWidth: 0 }}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </motion.div>
      </div>
    </div>
  );
}
