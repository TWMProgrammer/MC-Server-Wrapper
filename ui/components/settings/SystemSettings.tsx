import { Coffee, Trash2, Download, CheckCircle2, Loader2, Check, AlertCircle } from 'lucide-react'
import { motion } from 'framer-motion'
import { useState, useEffect } from 'react'
import { cn } from '../../utils'
import { AppSettings, ManagedJavaVersion } from '../../hooks/useAppSettings'
import { Section } from './SettingsShared'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToast } from '../../hooks/useToast'

interface SystemSettingsProps {
  settings: AppSettings;
  updateSettings: (newSettings: Partial<AppSettings>) => void;
}

export function SystemSettings({ settings, updateSettings }: SystemSettingsProps) {
  const { showToast } = useToast();
  const [downloadingVersion, setDownloadingVersion] = useState<number | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<{ downloaded: number, total: number } | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen('java_download_progress', (event: any) => {
        setDownloadProgress({
          downloaded: event.payload.downloaded as number,
          total: event.payload.total as number
        });
      });
    };

    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleDownloadJava = async (majorVersion: number) => {
    if (downloadingVersion !== null) return;

    setDownloadingVersion(majorVersion);
    setDownloadProgress(null);

    try {
      const newVersion = await invoke<ManagedJavaVersion>('download_java_version', { majorVersion });
      updateSettings({
        managed_java_versions: [
          ...settings.managed_java_versions.filter(v => v.id !== newVersion.id),
          newVersion
        ]
      });
      showToast(`Java ${majorVersion} installed successfully`, 'success');
    } catch (error) {
      console.error('Failed to download Java:', error);
      showToast(`Failed to download Java ${majorVersion}: ${error}`, 'error');
    } finally {
      setDownloadingVersion(null);
      setDownloadProgress(null);
    }
  };

  const handleDeleteJava = async (id: string, name: string) => {
    try {
      await invoke('delete_java_version', { id });
      updateSettings({
        managed_java_versions: settings.managed_java_versions.filter(v => v.id !== id)
      });
      showToast(`${name} deleted successfully`, 'success');
    } catch (error) {
      console.error('Failed to delete Java:', error);
      showToast(`Failed to delete Java: ${error}`, 'error');
    }
  };

  return (
    <div className="space-y-8">
      <Section title="Installed Versions" icon={Coffee}>
        <div className="space-y-3">
          {settings.managed_java_versions.length === 0 ? (
            <div className="p-8 text-center bg-black/5 dark:bg-white/5 rounded-2xl border border-dashed border-black/10 dark:border-white/10">
              <Coffee className="w-8 h-8 text-gray-400 mx-auto mb-3 opacity-20" />
              <div className="text-sm font-bold text-gray-500">No managed Java versions installed</div>
              <div className="text-xs text-gray-400 mt-1">Download a version below to get started</div>
            </div>
          ) : (
            settings.managed_java_versions.map((java) => (
              <div
                key={java.id}
                className="p-4 bg-black/5 dark:bg-white/5 rounded-2xl border border-black/5 dark:border-white/5 flex items-center justify-between group hover:border-primary/30 transition-all"
              >
                <div className="flex items-center gap-4">
                  <div className="p-3 bg-primary/10 rounded-xl text-primary">
                    <Coffee size={20} />
                  </div>
                  <div>
                    <div className="text-sm font-bold text-gray-900 dark:text-white">{java.name}</div>
                    <div className="text-[10px] text-gray-500 font-mono mt-0.5 flex items-center gap-2">
                      <span className="truncate max-w-[300px]">{java.path}</span>
                      <span className="px-1.5 py-0.5 bg-black/10 dark:bg-white/10 rounded uppercase tracking-wider text-[9px] font-bold">
                        v{java.version}
                      </span>
                    </div>
                  </div>
                </div>
                <button
                  onClick={() => handleDeleteJava(java.id, java.name)}
                  className="p-2 text-gray-400 hover:text-accent-rose hover:bg-accent-rose/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                  title="Delete version"
                >
                  <Trash2 size={18} />
                </button>
              </div>
            ))
          )}
        </div>
      </Section>

      <Section title="Download Java" icon={Download}>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {[8, 11, 17, 21].map((version) => {
            const isInstalled = settings.managed_java_versions.some(v => v.major_version === version);
            const isDownloading = downloadingVersion === version;

            return (
              <div
                key={version}
                className={cn(
                  "p-4 rounded-2xl border transition-all relative overflow-hidden flex flex-col justify-between h-32",
                  isInstalled
                    ? "bg-emerald-500/5 border-emerald-500/20"
                    : "bg-black/5 dark:bg-white/5 border-black/5 dark:border-white/5"
                )}
              >
                <div className="flex justify-between items-start">
                  <div>
                    <div className="text-sm font-bold text-gray-900 dark:text-white">Java {version}</div>
                    <div className="text-[10px] text-gray-500 mt-0.5">LTS Release • Adoptium</div>
                  </div>
                  {isInstalled ? (
                    <div className="p-1.5 bg-emerald-500 text-white rounded-lg shadow-glow-emerald/20">
                      <CheckCircle2 size={14} />
                    </div>
                  ) : isDownloading ? (
                    <div className="p-1.5 bg-primary text-white rounded-lg animate-pulse">
                      <Loader2 size={14} className="animate-spin" />
                    </div>
                  ) : (
                    <div className="p-1.5 bg-black/10 dark:bg-white/10 text-gray-400 rounded-lg">
                      <Coffee size={14} />
                    </div>
                  )}
                </div>

                {isDownloading && downloadProgress ? (
                  <div className="space-y-2">
                    <div className="flex justify-between text-[10px] font-bold uppercase tracking-wider text-primary">
                      <span>Downloading...</span>
                      <span>{Math.round((downloadProgress.downloaded / downloadProgress.total) * 100)}%</span>
                    </div>
                    <div className="h-1.5 bg-primary/10 rounded-full overflow-hidden">
                      <motion.div
                        className="h-full bg-primary"
                        initial={{ width: 0 }}
                        animate={{ width: `${(downloadProgress.downloaded / downloadProgress.total) * 100}%` }}
                      />
                    </div>
                  </div>
                ) : (
                  <button
                    onClick={() => handleDownloadJava(version)}
                    disabled={downloadingVersion !== null}
                    className={cn(
                      "w-full py-2 rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2",
                      isInstalled
                        ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-500/20"
                        : "bg-primary text-white shadow-glow-primary/20 hover:shadow-primary/40 active:scale-95 disabled:opacity-50 disabled:active:scale-100"
                    )}
                  >
                    {isInstalled ? (
                      <>
                        <Check size={14} />
                        Update Version
                      </>
                    ) : (
                      <>
                        <Download size={14} />
                        Download Java {version}
                      </>
                    )}
                  </button>
                )}
              </div>
            );
          })}
        </div>
        <div className="mt-4 p-4 bg-amber-500/5 border border-amber-500/10 rounded-2xl flex gap-3 items-start">
          <AlertCircle size={18} className="text-amber-500 shrink-0 mt-0.5" />
          <div className="text-[10px] text-amber-600 dark:text-amber-400 leading-relaxed font-medium">
            <strong>Note:</strong> Java versions are downloaded from the Adoptium (Eclipse Temurin) API and stored in the <code>java/</code> folder next to the application executable.
          </div>
        </div>
      </Section>
    </div>
  );
}
