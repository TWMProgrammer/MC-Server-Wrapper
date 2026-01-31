import { Terminal } from 'lucide-react'
import { cn } from '../utils'

interface ConsoleProps {
  logs: string[];
  consoleEndRef: React.RefObject<HTMLDivElement | null>;
  command: string;
  onCommandChange: (val: string) => void;
  onSendCommand: (e: React.FormEvent) => void;
  isFull?: boolean;
  onViewFull?: () => void;
}

export function Console({
  logs,
  consoleEndRef,
  command,
  onCommandChange,
  onSendCommand,
  isFull = false,
  onViewFull
}: ConsoleProps) {
  return (
    <div className={cn(
      "bg-[#000] rounded-lg border border-white/5 flex flex-col overflow-hidden",
      isFull ? "h-[calc(100vh-280px)]" : "h-80"
    )}>
      {!isFull && (
        <div className="bg-[#242424] px-4 py-2 border-b border-white/5 text-sm font-medium flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Terminal size={16} />
            Console
          </div>
          {onViewFull && (
            <button
              onClick={onViewFull}
              className="text-xs text-gray-500 hover:text-white transition-colors"
            >
              View Full Console
            </button>
          )}
        </div>
      )}
      <div className={cn(
        "flex-1 p-4 font-mono text-gray-400 overflow-y-auto space-y-0.5",
        isFull ? "text-sm" : "text-xs"
      )}>
        {(logs || []).map((line, i) => (
          <div key={i} className={cn(
            line.includes('ERROR') ? 'text-red-400' :
              line.includes('WARN') ? 'text-yellow-400' :
                line.includes('INFO') ? 'text-gray-400' : 'text-gray-300'
          )}>
            {line}
          </div>
        ))}
        <div ref={consoleEndRef} />
        {(!logs || logs.length === 0) && (
          <div className="text-gray-600 italic">No logs yet. Start the server to see output.</div>
        )}
      </div>
      <form onSubmit={onSendCommand} className="p-2 bg-[#1a1a1a] border-t border-white/5">
        <input
          type="text"
          value={command}
          onChange={(e) => onCommandChange(e.target.value)}
          placeholder="Type a command..."
          className={cn(
            "w-full bg-transparent border-none focus:ring-0 focus:outline-none font-mono px-2",
            isFull ? "text-sm" : "text-xs"
          )}
          autoComplete="off"
        />
        <button type="submit" className="hidden" />
      </form>
    </div>
  )
}
