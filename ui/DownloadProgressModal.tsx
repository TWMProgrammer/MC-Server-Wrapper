import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { Download, Loader2 } from 'lucide-react'
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
          // Add a small delay before closing or showing finished state
          setTimeout(() => setIsFinished(true), 500);
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl overflow-hidden p-6">
        <div className="flex items-center gap-4 mb-6">
          <div className="w-12 h-12 rounded-full bg-blue-500/20 flex items-center justify-center">
            <Download className="text-blue-400" size={24} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-zinc-100">Downloading Instance</h3>
            <p className="text-sm text-zinc-400">{instanceName}</p>
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex justify-between text-sm">
            <span className="text-zinc-400">{progress?.message || 'Preparing download...'}</span>
            <span className="text-zinc-100 font-medium">{percentage}%</span>
          </div>

          <div className="h-2 w-full bg-zinc-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-300 ease-out"
              style={{ width: `${percentage}%` }}
            />
          </div>

          <div className="flex justify-between text-xs text-zinc-500">
            <span>{progress ? formatBytes(progress.current) : '0 B'}</span>
            <span>{progress?.total ? formatBytes(progress.total) : '--'}</span>
          </div>
        </div>

        {isFinished ? (
          <button
            onClick={onClose}
            className="w-full mt-8 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg font-medium transition-colors"
          >
            Close
          </button>
        ) : (
          <div className="mt-8 flex items-center justify-center gap-2 text-sm text-zinc-500 italic">
            <Loader2 size={16} className="animate-spin" />
            Please wait while we set up your server...
          </div>
        )}
      </div>
    </div>
  )
}
