import { HardDrive, Cpu } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { Instance, InstanceSettings } from '../types'
import { Select } from '../components/Select'

interface GeneralSettingsProps {
  instance: Instance;
  name: string;
  setName: (name: string) => void;
  settings: InstanceSettings;
  updateSetting: <K extends keyof InstanceSettings>(key: K, value: InstanceSettings[K]) => void;
}

export function GeneralSettings({ instance, name, setName, settings, updateSetting }: GeneralSettingsProps) {
  return (
    <div className="space-y-8">
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

        {/* Children components like IconSettings are passed or handled separately in parent */}
      </div>

      <div className="pt-4 border-t border-black/10 dark:border-white/10">
        {/* Resources */}
        <div className="space-y-4">
          <h3 className="text-lg font-bold flex items-center gap-2">
            <Cpu size={20} className="text-primary" />
            Resources
          </h3>
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium text-gray-500 dark:text-white/40">Minimum RAM</label>
                <div className="flex gap-2">
                  <input
                    type="number"
                    value={settings.min_ram}
                    onChange={(e) => updateSetting('min_ram', parseInt(e.target.value) || 0)}
                    className="w-full min-w-0 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                  />
                  <Select
                    value={settings.min_ram_unit}
                    onChange={(value) => updateSetting('min_ram_unit', value)}
                    options={[
                      { value: 'G', label: 'GB' },
                      { value: 'M', label: 'MB' }
                    ]}
                    className="w-28 shrink-0"
                    size="sm"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-sm font-medium text-gray-500 dark:text-white/40">Maximum RAM</label>
                <div className="flex gap-2">
                  <input
                    type="number"
                    value={settings.max_ram}
                    onChange={(e) => updateSetting('max_ram', parseInt(e.target.value) || 0)}
                    className="w-full min-w-0 bg-black/5 dark:bg-white/[0.05] border border-black/10 dark:border-white/10 rounded-xl py-2 px-4 focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                  />
                  <Select
                    value={settings.max_ram_unit}
                    onChange={(value) => updateSetting('max_ram_unit', value)}
                    options={[
                      { value: 'G', label: 'GB' },
                      { value: 'M', label: 'MB' }
                    ]}
                    className="w-28 shrink-0"
                    size="sm"
                  />
                </div>
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
    </div>
  )
}
