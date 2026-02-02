import { useState, useRef, useEffect } from 'react'
import { Terminal, Maximize2, Send, ChevronRight, Activity } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import Ansi from 'ansi-to-react'
import { cn } from '../utils'
import { useAppSettings } from '../hooks/useAppSettings'

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
  const { settings } = useAppSettings();
  const [isAtBottom, setIsAtBottom] = useState(true);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const handleScroll = () => {
    if (scrollContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef.current;
      // Use a small threshold (10px) to determine if we're at the bottom
      const atBottom = scrollHeight - scrollTop - clientHeight < 50;
      setIsAtBottom(atBottom);
    }
  };

  const scrollToBottom = () => {
    consoleEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    setIsAtBottom(true);
  };

  // Auto-scroll when logs change, but only if we were already at the bottom
  useEffect(() => {
    if (isAtBottom) {
      consoleEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, isAtBottom]);

  const formatLogLine = (line: string) => {
    const timestampMatch = line.match(/^\[\d{2}:\d{2}:\d{2}\]/);
    const timestamp = timestampMatch ? timestampMatch[0] : '';
    const rest = timestampMatch ? line.slice(timestamp.length) : line;

    let typeColor = 'text-gray-400';
    if (line.includes('ERROR') || line.includes('Exception')) typeColor = 'text-accent-rose font-bold';
    else if (line.includes('WARN')) typeColor = 'text-accent-amber font-bold';
    else if (line.includes('INFO')) {
      typeColor = settings.use_white_console_text
        ? 'text-gray-900 dark:text-white font-bold'
        : 'text-primary/80 font-bold';
    }

    return (
      <div className="flex gap-3 py-0.5 group hover:bg-black/5 dark:hover:bg-white/[0.02] transition-colors rounded px-2 -mx-2">
        {timestamp && <span className="text-gray-400 dark:text-white/20 shrink-0 select-none font-medium">{timestamp}</span>}
        <span className={cn("break-all leading-relaxed", typeColor)}>
          <Ansi>{rest}</Ansi>
        </span>
      </div>
    );
  };

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      className={cn(
        "bg-white dark:bg-black/80 backdrop-blur-md rounded-2xl border border-black/10 dark:border-white/5 flex flex-col overflow-hidden shadow-2xl ring-1 ring-black/5 dark:ring-white/10 transition-all duration-300",
        isFull ? "h-full" : "h-96"
      )}
    >
      <div className="bg-black/[0.03] dark:bg-white/[0.03] px-6 py-3 border-b border-black/10 dark:border-white/5 text-sm font-bold flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex gap-1.5 mr-2">
            <div className="w-3 h-3 rounded-full bg-accent-rose/50" />
            <div className="w-3 h-3 rounded-full bg-accent-amber/50" />
            <div className="w-3 h-3 rounded-full bg-accent-emerald/50" />
          </div>
          <Terminal size={18} className="text-primary" />
          <span className="tracking-tight text-gray-900 dark:text-white">Live Server Console</span>
        </div>
        {!isFull && onViewFull && (
          <button
            onClick={onViewFull}
            className="p-1.5 hover:bg-black/5 dark:hover:bg-white/10 rounded-lg text-gray-400 hover:text-gray-900 dark:hover:text-white transition-all group"
            title="Expand Console"
          >
            <Maximize2 size={16} className="group-hover:scale-110 transition-transform" />
          </button>
        )}
      </div>

      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className={cn(
          "flex-1 p-6 font-mono overflow-y-auto no-scrollbar bg-black/5 dark:bg-black/40",
          isFull ? "text-sm" : "text-[13px]"
        )}
      >
        {logs && logs.length > 0 ? (
          logs.map((line, i) => (
            <div key={i}>{formatLogLine(line)}</div>
          ))
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-white/20 space-y-4">
            <Terminal size={48} className="opacity-10" />
            <p className="italic text-sm">No live data. The terminal is standing by...</p>
          </div>
        )}
        <div ref={consoleEndRef} />
      </div>

      <form onSubmit={onSendCommand} className="px-4 py-3 bg-black/[0.02] dark:bg-white/[0.02] border-t border-black/10 dark:border-white/5 flex items-center gap-3">
        <div className="text-primary">
          <ChevronRight size={18} />
        </div>
        <div className="flex-1 relative flex items-center">
          <input
            type="text"
            value={command}
            onChange={(e) => onCommandChange(e.target.value)}
            placeholder="Enter server command..."
            className={cn(
              "w-full bg-transparent border-none focus:ring-0 focus:outline-none font-mono text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20 transition-all",
              isFull ? "text-sm" : "text-[13px]",
              !isAtBottom && "pr-20"
            )}
            autoComplete="off"
          />
          <AnimatePresence>
            {!isAtBottom && (
              <motion.button
                initial={{ opacity: 0, x: 10, scale: 0.9 }}
                animate={{ opacity: 1, x: 0, scale: 1 }}
                exit={{ opacity: 0, x: 10, scale: 0.9 }}
                type="button"
                onClick={scrollToBottom}
                className="absolute right-0 flex items-center gap-1.5 px-3 py-1 bg-primary text-white text-[10px] font-black uppercase tracking-widest rounded-full shadow-glow-primary hover:scale-105 active:scale-95 transition-all z-10"
              >
                <Activity size={12} className="animate-pulse" />
                Live
              </motion.button>
            )}
          </AnimatePresence>
        </div>
        <button
          type="submit"
          disabled={!command.trim()}
          className="p-2 bg-primary/10 hover:bg-primary/20 text-primary rounded-lg transition-all disabled:opacity-0"
        >
          <Send size={16} />
        </button>
      </form>
    </motion.div>
  )
}
