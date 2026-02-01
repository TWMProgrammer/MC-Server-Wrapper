import { useState, useEffect } from 'react'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import {
  Settings,
  Shield,
  Terminal,
  RefreshCw,
  Save,
  Upload,
  Trash2,
  HardDrive,
  Cpu,
  Network,
  Play,
  Check,
  AlertCircle
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from './utils'
import { useToast } from './hooks/useToast'
import { Instance, InstanceSettings, LaunchMethod, CrashHandlingMode } from './types'
import { Select } from './components/Select'

interface InstanceSettingsTabProps {
  instance: Instance;
  onUpdate?: () => void;
}

type SettingsSubTab = 'general' | 'advanced' | 'crash' | 'update';

export function InstanceSettingsTab({ instance, onUpdate }: InstanceSettingsTabProps) {
  const [activeSubTab, setActiveSubTab] = useState<SettingsSubTab>('general')
  const [settings, setSettings] = useState<InstanceSettings>(instance.settings)
  const [name, setName] = useState(instance.name)
  const [tempIconPath, setTempIconPath] = useState<string | undefined>(instance.settings.icon_path)
  const [saving, setSaving] = useState(false)
  const [batFiles, setBatFiles] = useState<string[]>([])
  const [startupPreview, setStartupPreview] = useState<string>('')
  const [showPreview, setShowPreview] = useState(false)
  const [updatingJar, setUpdatingJar] = useState(false)
  const { showToast } = useToast()

  // Update local state when instance changes
  useEffect(() => {
    setSettings(instance.settings)
    setName(instance.name)
    setTempIconPath(instance.settings.icon_path)
    loadBatFiles()
  }, [instance])

  const loadBatFiles = async () => {
    try {
      const files = await invoke<string[]>('list_bat_files', { instanceId: instance.id })
      setBatFiles(files)
    } catch (err) {
      console.error('Failed to load bat files:', err)
    }
  }

  const handleBrowseJava = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Java Executable',
          extensions: ['exe', 'bin', '*']
        }]
      })
      if (selected && typeof selected === 'string') {
        updateSetting('java_path_override', selected)
      }
    } catch (err) {
      console.error('Failed to open file dialog:', err)
      showToast('Failed to open file dialog', 'error')
    }
  }

  const handleViewPreview = async () => {
    try {
      const preview = await invoke<string>('get_startup_preview', {
        instanceId: instance.id,
        settings: settings
      })
      setStartupPreview(preview)
      setShowPreview(true)
    } catch (err) {
      showToast(`Error generating preview: ${err}`, 'error')
    }
  }

  const handleIconUpload = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Images',
          extensions: ['png', 'jpg', 'jpeg', 'webp']
        }]
      })

      if (selected && typeof selected === 'string') {
        setTempIconPath(selected)
        updateSetting('icon_path', selected)
        showToast('Icon updated locally. Click Save to persist.', 'info')
      }
    } catch (err) {
      showToast(`Error selecting icon: ${err}`, 'error')
    }
  }

  const handleUpdateJar = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Minecraft Server JAR',
          extensions: ['jar']
        }]
      })

      if (selected && typeof selected === 'string') {
        setUpdatingJar(true)
        await invoke('update_instance_jar', {
          instanceId: instance.id,
          sourcePath: selected
        })
        showToast('Server JAR updated successfully', 'success')
      }
    } catch (err) {
      console.error('Failed to update JAR:', err)
      showToast(`Error: ${err}`, 'error')
    } finally {
      setUpdatingJar(false)
    }
  }

  const handleSave = async () => {
    if (!name.trim()) {
      showToast('Server name cannot be empty', 'error')
      return
    }

    if (settings.port < 1 || settings.port > 65535) {
      showToast('Invalid port number', 'error')
      return
    }

    setSaving(true)
    try {
      await invoke('update_instance_settings', {
        instanceId: instance.id,
        name: name !== instance.name ? name : undefined,
        settings: settings
      })

      showToast('Settings saved successfully', 'success')
      if (onUpdate) onUpdate()
    } catch (err) {
      console.error('Failed to save settings:', err)
      showToast(`Error: ${err}`, 'error')
    } finally {
      setSaving(false)
    }
  }

  const updateSetting = <K extends keyof InstanceSettings>(key: K, value: InstanceSettings[K]) => {
    setSettings(prev => ({ ...prev, [key]: value }))
  }

  const tabs = [
    { id: 'general', label: 'General', icon: Settings },
    { id: 'advanced', label: 'Advanced', icon: Terminal },
    { id: 'crash', label: 'Crash Handling', icon: Shield },
    { id: 'update', label: 'Update Server', icon: RefreshCw },
  ]

  return (
    <div className="flex flex-col h-full space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4 overflow-x-auto no-scrollbar pb-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveSubTab(tab.id as SettingsSubTab)}
              className={cn(
                "flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all whitespace-nowrap",
                activeSubTab === tab.id
                  ? "bg-primary text-white shadow-lg shadow-primary/20"
                  : "bg-black/5 dark:bg-white/5 text-gray-600 dark:text-white/60 hover:bg-black/10 dark:hover:bg-white/10"
              )}
            >
              <tab.icon size={16} />
              {tab.label}
            </button>
          ))}
        </div>

        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={handleSave}
          disabled={saving}
          className="flex items-center gap-2 px-6 py-2 bg-accent-emerald text-white rounded-xl font-bold shadow-lg shadow-accent-emerald/20 hover:shadow-accent-emerald/40 transition-all disabled:opacity-50"
        >
          {saving ? <RefreshCw size={18} className="animate-spin" /> : <Save size={18} />}
          Save Changes
        </motion.button>
      </div>

      <div className="flex-1 bg-black/5 dark:bg-white/[0.02] border border-black/10 dark:border-white/10 rounded-2xl p-6 overflow-y-auto">
        <AnimatePresence mode="wait">
          {activeSubTab === 'general' && (
            <motion.div
              key="general"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-8"
            >
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                {/* Server Identity */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold flex items-center gap-2">
                    <HardDrive size={20} className="text-primary" />
                    Server Identity
                  </h3>
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-gray-500 dark:text-white/40">Server Name</label>
                      <input
                        type="text"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        className="w-full bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                        placeholder="My Awesome Server"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-gray-500 dark:text-white/40">Description</label>
                      <textarea
                        value={settings.description || ''}
                        onChange={(e) => updateSetting('description', e.target.value)}
                        className="w-full bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-h-[100px] resize-none"
                        placeholder="A brief description of your server..."
                      />
                    </div>
                  </div>
                </div>

                {/* Server Icon */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold flex items-center gap-2">
                    <Upload size={20} className="text-primary" />
                    Server Icon
                  </h3>
                  <div className="flex items-start gap-6">
                    <div className="w-32 h-32 bg-black/5 dark:bg-white/[0.05] border-2 border-dashed border-black/10 dark:border-white/10 rounded-2xl flex items-center justify-center overflow-hidden group relative">
                      {tempIconPath ? (
                        <img
                          key={tempIconPath}
                          src={convertFileSrc(tempIconPath)}
                          alt="Server Icon"
                          className="w-full h-full object-cover"
                          onError={(e) => {
                            console.error("Failed to load icon:", tempIconPath);
                            // Fallback if image fails to load
                            e.currentTarget.style.display = 'none';
                          }}
                        />
                      ) : (
                        <HardDrive size={32} className="text-gray-400 dark:text-white/20" />
                      )}
                      <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                        <button
                          onClick={handleIconUpload}
                          className="p-2 bg-white/10 hover:bg-white/20 rounded-lg transition-colors"
                        >
                          <Upload size={18} className="text-white" />
                        </button>
                        {tempIconPath && (
                          <button
                            onClick={() => {
                              setTempIconPath(undefined)
                              updateSetting('icon_path', undefined)
                            }}
                            className="p-2 bg-accent-rose/20 hover:bg-accent-rose/40 rounded-lg transition-colors"
                          >
                            <Trash2 size={18} className="text-accent-rose" />
                          </button>
                        )}
                      </div>
                    </div>
                    <div className="flex-1 space-y-2">
                      <p className="text-sm text-gray-500 dark:text-white/40">
                        Upload a custom icon for your server. Recommended size is 64x64 or 128x128.
                      </p>
                      <button
                        onClick={handleIconUpload}
                        className="text-sm font-bold text-primary hover:underline"
                      >
                        Upload Image
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8 pt-4 border-t border-black/10 dark:border-white/10">
                {/* Resources */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold flex items-center gap-2">
                    <Cpu size={20} className="text-primary" />
                    Resources
                  </h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-gray-500 dark:text-white/40">Allocated RAM</label>
                      <div className="flex gap-2">
                        <input
                          type="number"
                          value={settings.ram}
                          onChange={(e) => updateSetting('ram', parseInt(e.target.value) || 0)}
                          className="flex-1 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                        />
                        <Select
                          value={settings.ram_unit}
                          onChange={(value) => updateSetting('ram_unit', value)}
                          options={[
                            { value: 'GB', label: 'GB' },
                            { value: 'MB', label: 'MB' }
                          ]}
                          className="w-24"
                          size="sm"
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-gray-500 dark:text-white/40">Server Port</label>
                      <input
                        type="number"
                        value={settings.port}
                        onChange={(e) => updateSetting('port', parseInt(e.target.value) || 25565)}
                        className="w-full bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                      />
                    </div>
                  </div>
                </div>

                {/* Automation */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold flex items-center gap-2">
                    <Network size={20} className="text-primary" />
                    Automation & Safety
                  </h3>
                  <div className="space-y-4">
                    <label className="flex items-center gap-3 p-3 bg-black/5 dark:bg-white/[0.03] rounded-xl cursor-pointer hover:bg-black/10 dark:hover:bg-white/5 transition-colors">
                      <input
                        type="checkbox"
                        checked={settings.force_save_all}
                        onChange={(e) => updateSetting('force_save_all', e.target.checked)}
                        className="w-5 h-5 rounded-lg border-black/10 dark:border-white/10 text-primary focus:ring-primary"
                      />
                      <div>
                        <p className="font-medium">Force 'save-all'</p>
                        <p className="text-xs text-gray-500 dark:text-white/40">Run 'save-all' command before stopping the server.</p>
                      </div>
                    </label>
                    <label className="flex items-center gap-3 p-3 bg-black/5 dark:bg-white/[0.03] rounded-xl cursor-pointer hover:bg-black/10 dark:hover:bg-white/5 transition-colors">
                      <input
                        type="checkbox"
                        checked={settings.autostart}
                        onChange={(e) => updateSetting('autostart', e.target.checked)}
                        className="w-5 h-5 rounded-lg border-black/10 dark:border-white/10 text-primary focus:ring-primary"
                      />
                      <div>
                        <p className="font-medium">Autostart</p>
                        <p className="text-xs text-gray-500 dark:text-white/40">Automatically start this server when the application launches.</p>
                      </div>
                    </label>
                  </div>
                </div>
              </div>

              <div className="space-y-2 pt-4 border-t border-black/10 dark:border-white/10">
                <label className="text-sm font-medium text-gray-500 dark:text-white/40">Instance Folder Path</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={instance.path}
                    readOnly
                    className="flex-1 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 text-gray-400 cursor-not-allowed"
                  />
                  <button
                    onClick={() => invoke('open_instance_folder', { instanceId: instance.id })}
                    className="px-4 py-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors text-sm font-medium"
                  >
                    Open Folder
                  </button>
                </div>
              </div>
            </motion.div>
          )}

          {activeSubTab === 'advanced' && (
            <motion.div
              key="advanced"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-8"
            >
              <div className="grid grid-cols-1 gap-8">
                {/* Java Path Override */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold flex items-center gap-2">
                    <Cpu size={20} className="text-primary" />
                    Java Configuration
                  </h3>
                  <div className="space-y-2">
                    <label className="text-sm font-medium text-gray-500 dark:text-white/40">Java Path Override (Optional)</label>
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={settings.java_path_override || ''}
                        onChange={(e) => updateSetting('java_path_override', e.target.value || undefined)}
                        className="flex-1 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                        placeholder="Default System Java"
                      />
                      <button
                        onClick={handleBrowseJava}
                        className="px-4 py-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors text-sm font-medium"
                        title="Select Java Executable"
                      >
                        Browse
                      </button>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-white/40">Leave empty to use the system's default Java or the version recommended for this Minecraft version.</p>
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
                          <div className="flex gap-2">
                            <select
                              value={settings.bat_file || ''}
                              onChange={(e) => updateSetting('bat_file', e.target.value || undefined)}
                              className="flex-1 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                            >
                              <option value="">Select a file...</option>
                              {batFiles.map(file => (
                                <option key={file} value={file}>{file}</option>
                              ))}
                            </select>
                            <button
                              onClick={loadBatFiles}
                              className="p-2 bg-black/5 dark:bg-white/5 hover:bg-black/10 dark:hover:bg-white/10 rounded-xl transition-colors"
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
            </motion.div>
          )}

          {activeSubTab === 'crash' && (
            <motion.div
              key="crash"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-6"
            >
              <div className="space-y-4">
                <h3 className="text-lg font-bold flex items-center gap-2">
                  <Shield size={20} className="text-primary" />
                  Crash Handling Mode
                </h3>
                <p className="text-sm text-gray-500 dark:text-white/40">
                  Configure how the wrapper should handle server stops and crashes.
                </p>
              </div>

              <div className="grid grid-cols-1 gap-4">
                {(['Nothing', 'Elevated', 'Aggressive'] as CrashHandlingMode[]).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => updateSetting('crash_handling', mode)}
                    className={cn(
                      "flex items-start gap-4 p-4 rounded-2xl border-2 transition-all text-left",
                      settings.crash_handling === mode
                        ? "bg-primary/5 border-primary shadow-lg shadow-primary/5"
                        : "bg-black/5 dark:bg-white/[0.02] border-transparent hover:border-black/10 dark:hover:border-white/10"
                    )}
                  >
                    <div className={cn(
                      "w-6 h-6 rounded-full border-2 flex items-center justify-center mt-1 shrink-0",
                      settings.crash_handling === mode ? "border-primary" : "border-gray-300 dark:border-white/20"
                    )}>
                      {settings.crash_handling === mode && <div className="w-3 h-3 rounded-full bg-primary" />}
                    </div>
                    <div>
                      <p className="font-bold text-lg">
                        {mode === 'Nothing' && 'Nothing (Default)'}
                        {mode === 'Elevated' && 'Elevated'}
                        {mode === 'Aggressive' && 'Aggressive'}
                      </p>
                      <p className="text-sm text-gray-500 dark:text-white/60">
                        {mode === 'Nothing' && 'No automatic restart. If the server stops for any reason, it stays stopped.'}
                        {mode === 'Elevated' && 'Restart if the server stops unexpectedly (non-zero exit code) or is stopped by non-wrapper input.'}
                        {mode === 'Aggressive' && 'Always restart the server whenever it stops, unless manually stopped through the wrapper.'}
                      </p>
                    </div>
                  </button>
                ))}
              </div>

              <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-2xl flex gap-4 items-start">
                <div className="p-2 bg-amber-500/20 rounded-lg text-amber-500 shrink-0">
                  <Shield size={20} />
                </div>
                <div className="space-y-1">
                  <p className="text-sm font-bold text-amber-600 dark:text-amber-400">Important Note</p>
                  <p className="text-xs text-amber-600/80 dark:text-amber-400/60 leading-relaxed">
                    Crash handling is only active while the wrapper is running. If you close the wrapper, the server will not be automatically restarted.
                  </p>
                </div>
              </div>
            </motion.div>
          )}

          {activeSubTab === 'update' && (
            <motion.div
              key="update"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-8"
            >
              <div className="space-y-4">
                <h3 className="text-lg font-bold flex items-center gap-2">
                  <RefreshCw size={20} className="text-primary" />
                  Update Server JAR
                </h3>
                <p className="text-sm text-gray-500 dark:text-white/40">
                  Replace the existing server JAR file with a new one. This is useful for manually updating to a new version or switching between server implementations (e.g., Paper to Purpur).
                </p>
              </div>

              <div
                className={cn(
                  "border-2 border-dashed rounded-3xl p-12 flex flex-col items-center justify-center gap-6 transition-all",
                  updatingJar
                    ? "bg-primary/5 border-primary animate-pulse"
                    : "bg-black/5 dark:bg-white/[0.02] border-black/10 dark:border-white/10 hover:border-primary/50"
                )}
              >
                <div className={cn(
                  "w-20 h-20 rounded-2xl flex items-center justify-center transition-colors",
                  updatingJar ? "bg-primary/20 text-primary" : "bg-black/5 dark:bg-white/5 text-gray-400"
                )}>
                  {updatingJar ? (
                    <RefreshCw size={40} className="animate-spin" />
                  ) : (
                    <Upload size={40} />
                  )}
                </div>

                <div className="text-center space-y-2">
                  <p className="text-xl font-bold">
                    {updatingJar ? 'Updating Server JAR...' : 'Select a new JAR file'}
                  </p>
                  <p className="text-sm text-gray-500 dark:text-white/40 max-w-xs mx-auto">
                    The selected file will be copied to the instance folder and renamed to <code>server.jar</code>.
                  </p>
                </div>

                <button
                  onClick={handleUpdateJar}
                  disabled={updatingJar}
                  className="px-8 py-3 bg-primary text-white rounded-xl font-bold shadow-lg shadow-primary/20 hover:shadow-primary/40 transition-all disabled:opacity-50"
                >
                  {updatingJar ? 'Please wait...' : 'Select JAR File'}
                </button>
              </div>

              <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-2xl flex gap-4 items-start">
                <div className="p-2 bg-blue-500/20 rounded-lg text-blue-500 shrink-0">
                  <AlertCircle size={20} />
                </div>
                <div className="space-y-1">
                  <p className="text-sm font-bold text-blue-600 dark:text-blue-400">Back up your data!</p>
                  <p className="text-xs text-blue-600/80 dark:text-blue-400/60 leading-relaxed">
                    It is highly recommended to create a backup of your instance before replacing the server JAR, especially when changing server types.
                  </p>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  )
}
