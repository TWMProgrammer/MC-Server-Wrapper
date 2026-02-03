import { Upload, HardDrive, Trash2 } from 'lucide-react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useToast } from '../hooks/useToast'
import { InstanceSettings } from '../types'

interface IconSettingsProps {
  tempIconPath: string | undefined;
  setTempIconPath: (path: string | undefined) => void;
  updateSetting: <K extends keyof InstanceSettings>(key: K, value: InstanceSettings[K]) => void;
}

export function IconSettings({ tempIconPath, setTempIconPath, updateSetting }: IconSettingsProps) {
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
    </div>
  )
}
