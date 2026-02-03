import { Cpu, Play, RefreshCw, Trash2 } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { InstanceSettings } from '../types'
import { Select } from '../components/Select'
import { cn } from '../utils'

interface JVMOptionsProps {
  settings: InstanceSettings;
  appSettings: any;
  updateSetting: <K extends keyof InstanceSettings>(key: K, value: InstanceSettings[K]) => void;
  handleBrowseJava: () => Promise<void>;
  handleViewPreview: () => Promise<void>;
  showPreview: boolean;
  setShowPreview: (show: boolean) => void;
  startupPreview: string;
  batFiles: string[];
  loadBatFiles: () => Promise<void>;
  showToast: (message: string, type: 'success' | 'error' | 'info') => void;
}

export function JVMOptions({
  settings,
  appSettings,
  updateSetting,
  handleBrowseJava,
  handleViewPreview,
  showPreview,
  setShowPreview,
  startupPreview,
  batFiles,
  loadBatFiles,
  showToast
}: JVMOptionsProps) {
  return (
    <div className="space-y-8">
      <div className="grid grid-cols-1 gap-8">
        {/* Java Path Override */}
        <div className="space-y-4">
          <h3 className="text-lg font-bold flex items-center gap-2">
            <Cpu size={20} className="text-primary" />
            Java Configuration
          </h3>
          <div className="space-y-2">
            <label className="text-sm font-medium text-gray-500 dark:text-white/40">Java Version</label>
            <div className="flex gap-2">
              <Select
                value={(() => {
                  if (!settings.java_path_override) return 'default';
                  const isManaged = appSettings.managed_java_versions.some((v: any) => v.id === settings.java_path_override);
                  return isManaged ? settings.java_path_override : 'custom';
                })()}
                onChange={(value) => {
                  if (value === 'default') {
                    updateSetting('java_path_override', undefined);
                  } else if (value === 'custom') {
                    handleBrowseJava();
                  } else {
                    updateSetting('java_path_override', value);
                  }
                }}
                options={[
                  { value: 'default', label: 'System Default (java)' },
                  ...appSettings.managed_java_versions.map((v: any) => ({
                    value: v.id,
                    label: `${v.name} (Managed)`
                  })),
                  {
                    value: 'custom',
                    label: settings.java_path_override && !appSettings.managed_java_versions.some((v: any) => v.id === settings.java_path_override)
                      ? `Custom: ${settings.java_path_override.split(/[\\/]/).pop()}`
                      : 'Custom...'
                  }
                ]}
                className="flex-1"
              />
              {settings.java_path_override && !appSettings.managed_java_versions.some((v: any) => v.id === settings.java_path_override) && (
                <button
                  onClick={handleBrowseJava}
                  className="px-4 py-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors text-sm font-medium"
                  title="Change Custom Java Path"
                >
                  Change
                </button>
              )}
            </div>
            {settings.java_path_override && !appSettings.managed_java_versions.some((v: any) => v.id === settings.java_path_override) && (
              <p className="text-[10px] font-mono text-gray-400 truncate max-w-full" title={settings.java_path_override}>
                Path: {settings.java_path_override}
              </p>
            )}
            <p className="text-xs text-gray-500 dark:text-white/40">Select a managed Java version or provide a custom path to a Java executable.</p>
          </div>
        </div>

        {/* Launch Method */}
        <div className="space-y-4 pt-4 border-t border-black/10 dark:border-white/10">
          <h3 className="text-lg font-bold flex items-center gap-2">
            <Play size={20} className="text-primary" />
            Launch Method
          </h3>

          <div className="flex gap-4 p-1 bg-black/5 dark:bg-white/5 rounded-2xl w-fit">
            <button
              onClick={() => updateSetting('launch_method', 'StartupLine')}
              className={cn(
                "px-6 py-2 rounded-xl text-sm font-bold transition-all",
                settings.launch_method === 'StartupLine'
                  ? "bg-white dark:bg-white/10 shadow-sm text-primary"
                  : "text-gray-500 hover:text-gray-700 dark:hover:text-white/80"
              )}
            >
              Startup Line
            </button>
            <button
              onClick={() => updateSetting('launch_method', 'BatFile')}
              className={cn(
                "px-6 py-2 rounded-xl text-sm font-bold transition-all",
                settings.launch_method === 'BatFile'
                  ? "bg-white dark:bg-white/10 shadow-sm text-primary"
                  : "text-gray-500 hover:text-gray-700 dark:hover:text-white/80"
              )}
            >
              Batch File
            </button>
          </div>

          <AnimatePresence mode="wait">
            {settings.launch_method === 'StartupLine' ? (
              <motion.div
                key="startup-line"
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                className="space-y-2 overflow-hidden"
              >
                <label className="text-sm font-medium text-gray-500 dark:text-white/40">Startup Command</label>
                <textarea
                  value={settings.startup_line}
                  onChange={(e) => updateSetting('startup_line', e.target.value)}
                  className="w-full bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all font-mono text-sm min-h-[80px]"
                  placeholder="java -Xmx{ram}{unit} -jar server.jar nogui"
                />
                <div className="flex justify-between items-center">
                  <p className="text-xs text-gray-500 dark:text-white/40">
                    Use <code>{'{ram}'}</code> and <code>{'{unit}'}</code> as placeholders for RAM settings.
                  </p>
                  <button
                    onClick={handleViewPreview}
                    className="text-xs font-bold text-primary hover:underline"
                  >
                    View Preview
                  </button>
                </div>

                <AnimatePresence>
                  {showPreview && (
                    <motion.div
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -10 }}
                      className="mt-2 p-3 bg-primary/10 border border-primary/20 rounded-xl relative group"
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-[10px] font-bold text-primary uppercase tracking-wider">Command Preview</span>
                        <button
                          onClick={() => setShowPreview(false)}
                          className="text-primary hover:text-primary/80"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                      <p className="text-xs font-mono text-gray-700 dark:text-white/80 break-all leading-relaxed">
                        {startupPreview}
                      </p>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            ) : (
              <motion.div
                key="bat-file"
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                className="space-y-4 overflow-hidden"
              >
                <div className="space-y-2">
                  <label className="text-sm font-medium text-gray-500 dark:text-white/40">Select Script File</label>
                  <div className="flex gap-2 items-center">
                    <Select
                      value={settings.bat_file || ''}
                      onChange={(value) => updateSetting('bat_file', value || undefined)}
                      options={batFiles.map(file => ({ value: file, label: file }))}
                      placeholder="Select a file..."
                      className="flex-1"
                    />
                    <button
                      onClick={loadBatFiles}
                      className="p-2.5 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors shrink-0"
                      title="Reload files"
                    >
                      <RefreshCw size={18} />
                    </button>
                  </div>
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={() => showToast('Edit File functionality coming soon', 'info')}
                    className="flex-1 px-4 py-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors text-sm font-medium"
                  >
                    Edit File
                  </button>
                  <button
                    onClick={() => showToast('Validate Script functionality coming soon', 'info')}
                    className="flex-1 px-4 py-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors text-sm font-medium"
                  >
                    Validate Script
                  </button>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  )
}
