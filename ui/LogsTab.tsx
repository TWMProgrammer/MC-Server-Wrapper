import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Search, FileText, Download, RefreshCw, History, Share } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from './utils'
import { useToast } from './hooks/useToast'

interface LogsTabProps {
  instanceId: string;
}

export function LogsTab({ instanceId }: LogsTabProps) {
  const [logs, setLogs] = useState<string[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const logsEndRef = useRef<HTMLDivElement>(null)
  const { showToast } = useToast()

  const fetchLogs = async () => {
    setLoading(true)
    setError(null)
    try {
      const content = await invoke<string>('read_latest_log', { instanceId })
      setLogs(content.split('\n').filter(line => line.trim() !== ''))
    } catch (err) {
      console.error('Failed to read logs:', err)
      setError('Failed to load latest.log. Make sure the server has been started at least once.')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchLogs()
  }, [instanceId])

  const filteredLogs = logs.filter(line =>
    line.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const downloadLogs = () => {
    const element = document.createElement("a");
    const file = new Blob([logs.join('\n')], { type: 'text/plain' });
    element.href = URL.createObjectURL(file);
    element.download = "latest.log";
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
  }

  const openExternal = async () => {
    try {
      await invoke('open_file_in_editor', {
        instanceId,
        relPath: 'logs/latest.log'
      })
    } catch (err) {
      console.error('Failed to open external editor:', err)
      showToast(`Error: ${err}`, 'error')
    }
  }

  const formatLogLine = (line: string) => {
    const timestampMatch = line.match(/^\[\d{2}:\d{2}:\d{2}\]/);
    const timestamp = timestampMatch ? timestampMatch[0] : '';
    const rest = timestampMatch ? line.slice(timestamp.length) : line;

    let typeColor = 'text-gray-400';
    if (line.includes('ERROR') || line.includes('Exception')) typeColor = 'text-accent-rose font-bold';
    else if (line.includes('WARN')) typeColor = 'text-accent-amber font-bold';
    else if (line.includes('INFO')) typeColor = 'text-primary/80 font-bold';

    return (
      <div className="flex gap-4 py-1 group hover:bg-black/5 dark:hover:bg-white/[0.02] transition-colors rounded-lg px-3 -mx-3">
        {timestamp && <span className="text-gray-400 dark:text-white/20 shrink-0 select-none font-mono text-[10px] pt-0.5">{timestamp}</span>}
        <span className={cn("break-all leading-relaxed", typeColor)}>{rest}</span>
      </div>
    );
  };

  return (
    <div className="flex flex-col h-[calc(100vh-280px)] space-y-6">
      <div className="flex items-center justify-between gap-6">
        <div className="relative flex-1 group">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={18} />
          <input
            type="text"
            placeholder="Search in logs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl py-3 pl-12 pr-4 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all placeholder:text-gray-400 dark:placeholder:text-white/20 font-medium"
          />
        </div>
        <div className="flex items-center gap-3">
          <motion.button
            whileHover={{ scale: 1.02, translateY: -2 }}
            whileTap={{ scale: 0.98 }}
            onClick={fetchLogs}
            disabled={loading}
            className="p-3 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] rounded-xl border border-black/10 dark:border-white/5 text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-all disabled:opacity-50 shadow-lg"
            title="Refresh logs"
          >
            <RefreshCw size={20} className={cn(loading && "animate-spin text-primary")} />
          </motion.button>
          <motion.button
            whileHover={{ scale: 1.02, translateY: -2 }}
            whileTap={{ scale: 0.98 }}
            onClick={openExternal}
            className="p-3 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] rounded-xl border border-black/10 dark:border-white/5 text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-all shadow-lg"
            title="Open in external editor"
          >
            <Share size={20} />
          </motion.button>
          <motion.button
            whileHover={{ scale: 1.02, translateY: -2 }}
            whileTap={{ scale: 0.98 }}
            onClick={downloadLogs}
            disabled={logs.length === 0}
            className="flex items-center gap-3 px-6 py-3 bg-primary/10 hover:bg-primary/20 rounded-xl border border-primary/20 text-primary hover:text-primary transition-all disabled:opacity-50 text-xs font-black uppercase tracking-widest shadow-lg"
          >
            <Download size={18} />
            Download
          </motion.button>
        </div>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex-1 bg-white dark:bg-black/80 backdrop-blur-md rounded-2xl border border-black/10 dark:border-white/5 overflow-hidden flex flex-col shadow-2xl ring-1 ring-black/5 dark:ring-white/10"
      >
        <div className="bg-black/[0.02] dark:bg-white/[0.03] px-6 py-3 border-b border-black/10 dark:border-white/5 text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/30 flex items-center gap-3">
          <div className="flex gap-1.5 mr-2">
            <div className="w-3 h-3 rounded-full bg-accent-rose/50" />
            <div className="w-3 h-3 rounded-full bg-accent-amber/50" />
            <div className="w-3 h-3 rounded-full bg-accent-emerald/50" />
          </div>
          <FileText size={16} className="text-primary" />
          <span className="text-gray-900 dark:text-white">latest.log</span>
          {searchQuery && (
            <motion.span
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="ml-auto text-primary"
            >
              Found {filteredLogs.length} matches
            </motion.span>
          )}
        </div>
        <div className="flex-1 p-6 font-mono text-sm overflow-y-auto custom-scrollbar bg-gray-50 dark:bg-[#050505]">
          <AnimatePresence mode="wait">
            {loading ? (
              <motion.div
                key="loading"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="h-full flex flex-col items-center justify-center text-gray-400 dark:text-white/20 gap-4"
              >
                <RefreshCw size={32} className="animate-spin text-primary" />
                <span className="text-[10px] font-black uppercase tracking-widest">Reading log file...</span>
              </motion.div>
            ) : error ? (
              <motion.div
                key="error"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="h-full flex flex-col items-center justify-center text-accent-rose/60 gap-4 text-center px-12"
              >
                <div className="w-16 h-16 rounded-full bg-accent-rose/10 flex items-center justify-center mb-2">
                  <History size={32} />
                </div>
                <p className="text-sm font-bold leading-relaxed">{error}</p>
                <button
                  onClick={fetchLogs}
                  className="mt-4 px-6 py-2 bg-accent-rose/20 hover:bg-accent-rose/30 rounded-lg text-xs font-black uppercase tracking-widest text-accent-rose transition-all"
                >
                  Try Again
                </button>
              </motion.div>
            ) : filteredLogs.length === 0 ? (
              <motion.div
                key="empty"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="h-full flex flex-col items-center justify-center text-gray-400 dark:text-white/10 gap-4"
              >
                <div className="w-16 h-16 rounded-full bg-black/5 dark:bg-white/[0.02] flex items-center justify-center mb-2">
                  <Search size={32} />
                </div>
                <span className="text-sm font-bold uppercase tracking-widest">
                  {searchQuery ? 'No matches found' : 'Log file is empty'}
                </span>
              </motion.div>
            ) : (
              <motion.div
                key="content"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="space-y-0.5"
              >
                {filteredLogs.map((line, i) => (
                  <div key={i}>
                    {formatLogLine(line)}
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </motion.div>
    </div>
  )
}
