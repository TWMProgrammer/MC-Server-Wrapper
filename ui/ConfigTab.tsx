import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, Info } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { TextEditor } from './components/TextEditor'
import { ConfigFile } from './config/types'
import { ConfigSidebar } from './config/ConfigSidebar'
import { ConfigControls } from './config/ConfigControls'
import { PropertyGrid } from './config/PropertyGrid'

interface ConfigTabProps {
  instanceId: string
}

export function ConfigTab({ instanceId }: ConfigTabProps) {
  const [properties, setProperties] = useState<Record<string, string>>({})
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [availableConfigs, setAvailableConfigs] = useState<ConfigFile[]>([])
  const [selectedConfig, setSelectedConfig] = useState<ConfigFile | null>(null)
  const [isRawEditing, setIsRawEditing] = useState(false)
  const [rawContent, setRawContent] = useState('')
  const [nestedConfig, setNestedConfig] = useState<any>(null)

  const fetchAvailableConfigs = async () => {
    try {
      const configs = await invoke<ConfigFile[]>('get_available_configs', { instanceId })
      setAvailableConfigs(configs)
      if (configs.length > 0 && !selectedConfig) {
        setSelectedConfig(configs[0])
      }
    } catch (err) {
      console.error('Failed to fetch available configs:', err)
    }
  }

  const fetchProperties = async () => {
    if (!selectedConfig) return
    setLoading(true)
    setError(null)
    try {
      const props = await invoke<Record<string, string>>('get_config_file', {
        instanceId,
        relPath: selectedConfig.path,
        format: selectedConfig.format
      })
      setProperties(props)

      if (selectedConfig.name === 'commands.yml') {
        const nested = await invoke<any>('get_config_value', {
          instanceId,
          relPath: selectedConfig.path,
          format: selectedConfig.format
        })
        setNestedConfig(nested)
      } else {
        setNestedConfig(null)
      }
    } catch (err) {
      setError(err as string)
    } finally {
      setLoading(false)
    }
  }

  const handleRawEdit = async () => {
    if (!selectedConfig) return
    try {
      const content = await invoke<string>('read_text_file', {
        instanceId,
        relPath: selectedConfig.path
      })
      setRawContent(content)
      setIsRawEditing(true)
    } catch (err) {
      setError(err as string)
    }
  }

  const handleRawSave = async (content: string) => {
    if (!selectedConfig) return
    try {
      await invoke('save_text_file', {
        instanceId,
        relPath: selectedConfig.path,
        content
      })
      setRawContent(content)
      fetchProperties()
    } catch (err) {
      setError(err as string)
    }
  }

  useEffect(() => {
    fetchAvailableConfigs()
  }, [instanceId])

  useEffect(() => {
    if (selectedConfig) {
      fetchProperties()
    }
  }, [selectedConfig, instanceId])

  const handleSave = async () => {
    if (!selectedConfig) return
    setSaving(true)
    setError(null)
    try {
      if (nestedConfig && selectedConfig.name === 'commands.yml') {
        await invoke('save_config_value', {
          instanceId,
          relPath: selectedConfig.path,
          format: selectedConfig.format,
          value: nestedConfig
        })
      } else {
        await invoke('save_config_file', {
          instanceId,
          relPath: selectedConfig.path,
          format: selectedConfig.format,
          properties
        })
      }
    } catch (err) {
      setError(err as string)
    } finally {
      setSaving(false)
    }
  }

  const handlePropertyChange = (key: string, value: string) => {
    setProperties(prev => ({ ...prev, [key]: value }))
  }

  const filteredKeys = Object.keys(properties)
    .filter(key => key.toLowerCase().includes(searchTerm.toLowerCase()))
    .sort()

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
        >
          <RefreshCw className="text-primary w-12 h-12 opacity-50" />
        </motion.div>
        <span className="text-gray-400 dark:text-white/40 font-medium tracking-wider uppercase text-xs">Loading properties...</span>
      </div>
    )
  }

  return (
    <div className="flex gap-8 h-full min-h-[600px]">
      <ConfigSidebar
        availableConfigs={availableConfigs}
        selectedConfig={selectedConfig}
        setSelectedConfig={setSelectedConfig}
      />

      <div className="flex-1 space-y-8 pb-8">
        <ConfigControls
          selectedConfig={selectedConfig}
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          onRefresh={fetchProperties}
          onRawEdit={handleRawEdit}
          onSave={handleSave}
          saving={saving}
        />

        <AnimatePresence>
          {isRawEditing && (
            <TextEditor
              title={selectedConfig?.name || 'Config Editor'}
              initialValue={rawContent}
              onSave={handleRawSave}
              onClose={() => setIsRawEditing(false)}
            />
          )}
        </AnimatePresence>

        <AnimatePresence mode="wait">
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="bg-accent-rose/10 border border-accent-rose/20 text-accent-rose p-5 rounded-2xl flex items-center gap-4"
            >
              <div className="w-10 h-10 rounded-full bg-accent-rose/20 flex items-center justify-center shrink-0">
                <Info size={20} />
              </div>
              <p className="text-sm font-medium">{error}</p>
            </motion.div>
          )}
        </AnimatePresence>

        <PropertyGrid
          selectedConfig={selectedConfig}
          nestedConfig={nestedConfig}
          setNestedConfig={setNestedConfig}
          filteredKeys={filteredKeys}
          properties={properties}
          onPropertyChange={handlePropertyChange}
        />
      </div>
    </div>
  )
}
