import { useState, useRef, useEffect, useLayoutEffect } from 'react'
import { Terminal, Maximize2, Send, ChevronRight, Activity, Check } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import Ansi from 'ansi-to-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { cn } from '../utils'
import { useAppSettings, AppSettings } from '../hooks/useAppSettings'

interface ConsoleProps {
  logs: string[];
  consoleEndRef: React.RefObject<HTMLDivElement | null>;
  command: string;
  onCommandChange: (val: string) => void;
  onSendCommand: (e: React.FormEvent) => void;
  isFull?: boolean;
  onViewFull?: () => void;
  settings?: AppSettings;
}

export function Console({
  logs,
  consoleEndRef,
  command,
  onCommandChange,
  onSendCommand,
  isFull = false,
  onViewFull,
  settings: propSettings
}: ConsoleProps) {
  const { settings: hookSettings } = useAppSettings();
  const settings = propSettings || hookSettings;
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [isWrapped, setIsWrapped] = useState(true);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isAutoScrolling = useRef(false);
  const LINE_HEIGHT = 18; // Approximate height of a log line

  useEffect(() => {
    if (scrollContainerRef.current) {
      setContainerHeight(scrollContainerRef.current.clientHeight);

      const resizeObserver = new ResizeObserver(entries => {
        for (let entry of entries) {
          setContainerHeight(entry.contentRect.height);
        }
      });

      resizeObserver.observe(scrollContainerRef.current);
      return () => resizeObserver.disconnect();
    }
  }, []);

  const handleScroll = () => {
    if (scrollContainerRef.current) {
      const { scrollTop: st, scrollHeight, clientHeight } = scrollContainerRef.current;
      setScrollTop(st);

      if (!isAutoScrolling.current) {
        // Use a threshold to determine if we're at the bottom
        const atBottom = scrollHeight - st - clientHeight < 100;
        setIsAtBottom(atBottom);
      }
    }
  };

  const scrollToBottom = (smooth = false) => {
    if (scrollContainerRef.current) {
      isAutoScrolling.current = true;
      const { scrollHeight } = scrollContainerRef.current;

      scrollContainerRef.current.scrollTo({
        top: scrollHeight,
        behavior: smooth ? 'smooth' : 'auto'
      });

      // We are definitely at the bottom now
      setIsAtBottom(true);

      // Reset auto-scrolling flag after a short delay
      setTimeout(() => {
        isAutoScrolling.current = false;
      }, smooth ? 500 : 50);
    }
  };

  // Auto-scroll when logs change
  useLayoutEffect(() => {
    if (isAtBottom) {
      scrollToBottom(false);
    }
  }, [logs]);

  // Handle manual scroll to bottom
  const handleManualScrollToBottom = () => {
    scrollToBottom(true);
  };

  const formatLogLine = (line: string) => {
    const timestampMatch = line.match(/^\[\d{2}:\d{2}:\d{2}\]/);
    const timestamp = timestampMatch ? timestampMatch[0] : '';
    const rest = timestampMatch ? line.slice(timestamp.length) : line;

    // Split text by URLs (http/https), excluding whitespace and ANSI escape codes
    const parts = rest.split(/(https?:\/\/[^\s\x1b]+)/g);

    let typeColor = settings.use_white_console_text
      ? 'text-gray-900 dark:text-white/90'
      : 'text-gray-400';

    if (line.includes('ERROR') || line.includes('Exception')) typeColor = 'text-accent-rose';
    else if (line.includes('WARN')) typeColor = 'text-accent-amber';
    else if (line.includes('INFO')) {
      typeColor = settings.use_white_console_text
        ? 'text-gray-900 dark:text-white/90'
        : 'text-primary/80';
    }

    return (
      <div className="flex gap-2 py-0 group hover:bg-black/5 dark:hover:bg-white/[0.02] transition-colors rounded px-2 -mx-2">
        {timestamp && <span className="text-gray-400 dark:text-white/20 shrink-0 select-none">{timestamp}</span>}
        <span className={cn("leading-tight", isWrapped ? "whitespace-pre-wrap break-all" : "whitespace-pre", typeColor)}>
          {parts.map((part, i) => {
            if (part.match(/^https?:\/\//)) {
              return (
                <a
                  key={i}
                  href={part}
                  onClick={async (e) => {
                    e.preventDefault();
                    try {
                      await openUrl(part);
                    } catch (err) {
                      console.error("Failed to open URL:", err);
                    }
                  }}
                  className="underline hover:text-blue-400 dark:hover:text-blue-400 cursor-pointer transition-colors"
                  title="Click to open in default browser"
                >
                  <Ansi>{part}</Ansi>
                </a>
              );
            }
            return <Ansi key={i}>{part}</Ansi>;
          })}
        </span>
      </div>
    );
  };

  const totalHeight = logs.length * LINE_HEIGHT;
  const startIndex = Math.max(0, Math.floor(scrollTop / LINE_HEIGHT) - 5);
  const endIndex = Math.min(logs.length, Math.ceil((scrollTop + containerHeight) / LINE_HEIGHT) + 5);
  const visibleLogs = logs.slice(startIndex, endIndex);
  const offsetTop = startIndex * LINE_HEIGHT;

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
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-2 cursor-pointer group select-none">
            <div className="relative flex items-center">
              <input
                type="checkbox"
                checked={isWrapped}
                onChange={(e) => setIsWrapped(e.target.checked)}
                className="peer sr-only"
              />
              <div className={cn(
                "w-3.5 h-3.5 rounded border transition-all flex items-center justify-center",
                isWrapped
                  ? "bg-primary border-primary"
                  : "border-gray-400 dark:border-white/30 group-hover:border-primary/50"
              )}>
                {isWrapped && <Check size={10} className="text-white" />}
              </div>
            </div>
            <span className={cn(
              "text-xs font-medium transition-colors",
              isWrapped ? "text-primary" : "text-gray-500 group-hover:text-gray-900 dark:group-hover:text-gray-300"
            )}>
              Wrap
            </span>
          </label>
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
      </div>

      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        className={cn(
          "flex-1 p-6 font-mono overflow-auto no-scrollbar bg-black/5 dark:bg-black/40 relative",
          isFull ? "text-sm" : "text-[13px]"
        )}
      >
        {logs && logs.length > 0 ? (
          isWrapped ? (
            <div className="flex flex-col">
              {logs.map((line, i) => (
                <div key={i} className="min-h-[18px]">
                  {formatLogLine(line)}
                </div>
              ))}
            </div>
          ) : (
            <div style={{ height: totalHeight, position: 'relative' }}>
              <div style={{ transform: `translateY(${offsetTop}px)`, position: 'absolute', top: 0, left: 0, right: 0 }}>
                {visibleLogs.map((line, i) => (
                  <div key={startIndex + i} style={{ height: LINE_HEIGHT, overflow: 'hidden' }}>
                    {formatLogLine(line)}
                  </div>
                ))}
              </div>
            </div>
          )
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
                onClick={handleManualScrollToBottom}
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
