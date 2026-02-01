import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { Download, Loader2, CheckCircle2, Info, ArrowRight } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from './utils'

interface ProgressPayload {
  instance_id: string;
  current: number;
  total: number;
  message: string;
}

interface DownloadProgressModalProps {
  isOpen: boolean;
  onClose: () => void;
  instanceId: string | null;
  instanceName: string;
}

export function DownloadProgressModal({ isOpen, onClose, instanceId, instanceName }: DownloadProgressModalProps) {
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  const [isFinished, setIsFinished] = useState(false);

  useEffect(() => {
    if (!isOpen || !instanceId) {
      setProgress(null);
      setIsFinished(false);
      return;
    }

    const unlisten = listen<ProgressPayload>('download-progress', (event) => {
      if (event.payload.instance_id === instanceId) {
        setProgress(event.payload);
        if (event.payload.current >= event.payload.total && event.payload.total > 0) {
          setTimeout(() => setIsFinished(true), 800);
        }
      }
    });

    return () => {
      unlisten.then(u => u());
    };
  }, [isOpen, instanceId]);

  if (!isOpen) return null;

  const percentage = progress?.total ? Math.round((progress.current / progress.total) * 100) : 0;
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center p-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/80 backdrop-blur-xl"
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 20 }}
            className="w-full max-w-md bg-white dark:bg-gray-950 border border-black/10 dark:border-white/10 rounded-3xl shadow-2xl overflow-hidden relative z-10 ring-1 ring-black/5 dark:ring-white/5"
          >
            <div className="p-8">
              <div className="flex items-center gap-6 mb-8">
                <div className={cn(
                  "w-16 h-16 rounded-2xl flex items-center justify-center transition-all duration-500 shadow-2xl",
                  isFinished ? "bg-emerald-500/20 text-emerald-400 rotate-[360deg]" : "bg-primary/20 text-primary animate-pulse"
                )}>
                  {isFinished ? <CheckCircle2 size={32} /> : <Download size={32} />}
                </div>
                <div>
                  <div className="text-[10px] font-black uppercase tracking-[0.2em] text-primary mb-1">Instance Setup</div>
                  <h3 className="text-xl font-black text-gray-900 dark:text-white">{isFinished ? 'Ready to Play!' : 'Downloading Files'}</h3>
                  <p className="text-sm font-medium text-gray-500 dark:text-white/40 truncate max-w-[200px]">{instanceName}</p>
                </div>
              </div>

              <div className="space-y-6">
                <div className="flex justify-between items-end">
                  <div className="space-y-1">
                    <span className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20">Current Task</span>
                    <div className="text-sm font-bold text-gray-700 dark:text-white/80 flex items-center gap-2">
                      {!isFinished && <Loader2 size={14} className="animate-spin text-primary" />}
                      {isFinished ? 'Installation Complete' : (progress?.message || 'Initializing...')}
                    </div>
                  </div>
                  <div className="text-2xl font-black font-mono tracking-tighter text-primary">
                    {percentage}%
                  </div>
                </div>

                <div className="h-3 w-full bg-black/5 dark:bg-white/5 rounded-full overflow-hidden p-0.5 border border-black/5 dark:border-white/5">
                  <motion.div
                    initial={{ width: 0 }}
                    animate={{ width: `${percentage}%` }}
                    className={cn(
                      "h-full rounded-full transition-all duration-300 shadow-glow-primary relative overflow-hidden",
                      isFinished ? "bg-emerald-500 shadow-glow-emerald" : "bg-primary"
                    )}
                  >
                    <motion.div
                      animate={{ x: ['-100%', '100%'] }}
                      transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                      className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent"
                    />
                  </motion.div>
                </div>

                <div className="flex justify-between items-center text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20 bg-black/[0.02] dark:bg-white/[0.02] p-3 rounded-xl border border-black/5 dark:border-white/5">
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500 dark:text-white/40">{progress ? formatBytes(progress.current) : '0 B'}</span>
                  </div>
                  <ArrowRight size={12} className="text-gray-300 dark:text-white/10" />
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500 dark:text-white/40">{progress?.total ? formatBytes(progress.total) : '--'}</span>
                  </div>
                </div>
              </div>

              <div className="mt-10">
                {isFinished ? (
                  <motion.button
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    whileHover={{ scale: 1.02, translateY: -2 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={onClose}
                    className="w-full py-4 bg-emerald-500 hover:bg-emerald-400 text-white rounded-2xl font-black uppercase tracking-widest text-xs transition-all shadow-glow-emerald"
                  >
                    Start Server Now
                  </motion.button>
                ) : (
                  <div className="flex items-center justify-center gap-3 text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">
                    <Info size={14} className="text-primary" />
                    Please keep this window open
                  </div>
                )}
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}
