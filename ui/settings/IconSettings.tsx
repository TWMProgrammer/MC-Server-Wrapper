import { Upload, HardDrive, Trash2, Network } from 'lucide-react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '../hooks/useToast'
import { InstanceSettings } from '../types'

interface IconSettingsProps {
  tempIconPath: string | undefined;
  setTempIconPath: (path: string | undefined) => void;
  settings: InstanceSettings;
  updateSetting: <K extends keyof InstanceSettings>(key: K, value: InstanceSettings[K]) => void;
}

export function IconSettings({ tempIconPath, setTempIconPath, settings, updateSetting }: IconSettingsProps) {
  const { showToast } = useToast()

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

  return (
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

      {/* Automation */}
      <div className="space-y-4 pt-8 border-t border-black/10 dark:border-white/10">
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
  )
}
