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
              <div className="flex-1 h-1.5 bg-black/5 dark:bg-white/5 rounded-full overflow-hidden transition-colors duration-300">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: stat.label.includes('Usage') ? (stat.label === 'CPU Usage' ? `${usage?.cpu_usage || 0}%` : '45%') : '0%' }}
                  className={`h-full ${stat.color.replace('text-', 'bg-')} opacity-60`}
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
          <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={history}>
                <defs>
                  <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff05" vertical={false} />
                <XAxis dataKey="timestamp" hide />
                <YAxis domain={settings.dynamic_graph_scaling ? ['auto', 'auto'] : [0, 100]} hide />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--surface)',
                    backdropFilter: 'blur(8px)',
                    border: '1px solid var(--border)',
                    borderRadius: '12px',
                    color: 'var(--foreground)'
                  }}
                  itemStyle={{ color: '#10b981' }}
                  labelStyle={{ display: 'none' }}
                />
                <Area
                  type="monotone"
                  dataKey="cpu_usage"
                  stroke="#10b981"
                  strokeWidth={3}
                  fillOpacity={1}
                  fill="url(#colorCpu)"
                  isAnimationActive={true}
                  animationDuration={1000}
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
          <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={history}>
                <defs>
                  <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff05" vertical={false} />
                <XAxis dataKey="timestamp" hide />
                <YAxis domain={settings.dynamic_graph_scaling ? ['auto', 'auto'] : [0, 'auto']} hide />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--surface)',
                    backdropFilter: 'blur(8px)',
                    border: '1px solid var(--border)',
                    borderRadius: '12px',
                    color: 'var(--foreground)'
                  }}
                  itemStyle={{ color: '#3b82f6' }}
                  labelStyle={{ display: 'none' }}
                  formatter={(value: number | undefined) => {
                    if (value === undefined) return ['0 MB', 'Memory'];
                    return [`${(value / 1024 / 1024).toFixed(0)} MB`, 'Memory'];
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="memory_usage"
                  stroke="#3b82f6"
                  strokeWidth={3}
                  fillOpacity={1}
                  fill="url(#colorMem)"
                  isAnimationActive={true}
                  animationDuration={1000}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </motion.div>
      </div>
    </div>
  );
}
