import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Search, FileText, Download, RefreshCw } from 'lucide-react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface LogsTabProps {
  instanceId: string;
}

export function LogsTab({ instanceId }: LogsTabProps) {
  const [logs, setLogs] = useState<string[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const logsEndRef = useRef<HTMLDivElement>(null)

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
    const file = new Blob([logs.join('\n')], {type: 'text/plain'});
    element.href = URL.createObjectURL(file);
    element.download = "latest.log";
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
  }

  return (
    <div className="flex flex-col h-[calc(100vh-280px)] space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
          <input
            type="text"
            placeholder="Search in logs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-[#242424] border border-white/5 rounded-md py-2 pl-10 pr-4 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 transition-all"
          />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={fetchLogs}
            disabled={loading}
            className="p-2 bg-[#242424] hover:bg-[#2d2d2d] rounded-md border border-white/5 text-gray-400 hover:text-white transition-all disabled:opacity-50"
            title="Refresh logs"
          >
            <RefreshCw size={18} className={cn(loading && "animate-spin")} />
          </button>
          <button
            onClick={downloadLogs}
            disabled={logs.length === 0}
            className="flex items-center gap-2 px-4 py-2 bg-[#242424] hover:bg-[#2d2d2d] rounded-md border border-white/5 text-gray-400 hover:text-white transition-all disabled:opacity-50 text-sm font-medium"
          >
            <Download size={18} />
            Download
          </button>
        </div>
      </div>

      <div className="flex-1 bg-black rounded-lg border border-white/5 overflow-hidden flex flex-col">
        <div className="bg-[#242424] px-4 py-2 border-b border-white/5 text-xs font-medium text-gray-500 flex items-center gap-2">
          <FileText size={14} />
          latest.log
          {searchQuery && (
            <span className="ml-auto text-blue-400">
              Found {filteredLogs.length} matches
            </span>
          )}
        </div>
        <div className="flex-1 p-4 font-mono text-xs overflow-y-auto space-y-0.5 custom-scrollbar">
          {loading ? (
            <div className="h-full flex items-center justify-center text-gray-500 italic">
              Loading logs...
            </div>
          ) : error ? (
            <div className="h-full flex items-center justify-center text-red-400 italic text-center px-10">
              {error}
            </div>
          ) : filteredLogs.length === 0 ? (
            <div className="h-full flex items-center justify-center text-gray-500 italic">
              {searchQuery ? 'No matches found.' : 'Log file is empty.'}
            </div>
          ) : (
            filteredLogs.map((line, i) => (
              <div key={i} className={cn(
                "whitespace-pre-wrap break-all",
                line.includes('ERROR') ? 'text-red-400' :
                line.includes('WARN') ? 'text-yellow-400' :
                line.includes('INFO') ? 'text-gray-400' : 'text-gray-300'
              )}>
                {line}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
