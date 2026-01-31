import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Save, RefreshCw, Search, Settings2, Info } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from './utils'
import { Select } from './components/Select'

interface ConfigTabProps {
  instanceId: string
}

export function ConfigTab({ instanceId }: ConfigTabProps) {
  const [properties, setProperties] = useState<Record<string, string>>({})
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [searchTerm, setSearchTerm] = useState('')

  const fetchProperties = async () => {
    setLoading(true)
    setError(null)
    try {
      const props = await invoke<Record<string, string>>('get_server_properties', { instanceId })
      setProperties(props)
    } catch (err) {
      setError(err as string)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchProperties()
  }, [instanceId])

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    try {
      await invoke('save_server_properties', { instanceId, properties })
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

  const renderInput = (key: string, value: string) => {
    const isBoolean = value === 'true' || value === 'false'
    const isNumeric = !isNaN(Number(value)) && value.trim() !== ''

    const inputClasses = "w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all hover:bg-black/10 dark:hover:bg-white/[0.05]"

    if (isBoolean) {
      return (
        <Select
          value={value}
          onChange={(newValue) => handlePropertyChange(key, newValue)}
          options={[
            { value: 'true', label: 'true' },
            { value: 'false', label: 'false' },
          ]}
        />
      )
    }

    if (isNumeric) {
      return (
        <input
          type="number"
          className={inputClasses}
          value={value}
          onChange={(e) => handlePropertyChange(key, e.target.value)}
        />
      )
    }

    return (
      <input
        type="text"
        className={inputClasses}
        value={value}
        onChange={(e) => handlePropertyChange(key, e.target.value)}
      />
    )
  }

  return (
    <div className="space-y-8 pb-8">
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
        <div className="relative flex-1 w-full max-w-xl group">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 dark:text-white/20 group-focus-within:text-primary transition-colors" size={20} />
          <input
            type="text"
            placeholder="Search server properties (e.g. motd, port, seeds)..."
            className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-2xl py-3.5 pl-12 pr-6 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-white/20"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <div className="flex gap-3 w-full md:w-auto">
          <motion.button
            whileHover={{
              scale: 1.02,
              translateY: -2,
              transition: { duration: 0.2, ease: "easeOut" }
            }}
            whileTap={{ scale: 0.98 }}
            onClick={fetchProperties}
            className="flex-1 md:flex-none flex items-center justify-center gap-2 px-6 py-3.5 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] border border-black/10 dark:border-white/10 rounded-2xl transition-all duration-200 text-sm font-bold uppercase tracking-widest text-gray-500 dark:text-white/60 hover:text-gray-900 dark:hover:text-white"
          >
            <RefreshCw size={18} />
            Refresh
          </motion.button>
          <motion.button
            whileHover={{
              scale: 1.02,
              translateY: -2,
              transition: { duration: 0.2, ease: "easeOut" }
            }}
            whileTap={{ scale: 0.98 }}
            onClick={handleSave}
            disabled={saving}
            className="flex-1 md:flex-none flex items-center justify-center gap-2 px-8 py-3.5 bg-primary hover:bg-primary-hover disabled:bg-black/5 dark:disabled:bg-white/5 disabled:text-gray-400 dark:disabled:text-white/20 disabled:cursor-not-allowed rounded-2xl transition-all duration-200 text-sm font-bold uppercase tracking-widest text-white shadow-glow-primary"
          >
            {saving ? (
              <RefreshCw className="animate-spin" size={18} />
            ) : (
              <Save size={18} />
            )}
            {saving ? 'Saving...' : 'Save Changes'}
          </motion.button>
        </div>
      </div>

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

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        <AnimatePresence mode="popLayout">
          {filteredKeys.map((key, index) => (
            <motion.div
              layout
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              transition={{ delay: index * 0.01 }}
              key={key}
              className="glass-panel p-5 rounded-2xl border border-black/5 dark:border-white/5 flex flex-col gap-3 group hover:border-primary/30 transition-all duration-200 hover:translate-y-[-2px] focus-within:z-20 relative"
            >
              <div className="flex items-center justify-between">
                <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors">
                  {key.replace(/-/g, ' ')}
                </label>
                <Settings2 size={14} className="text-gray-400 dark:text-white/10 group-hover:text-primary transition-colors" />
              </div>
              {renderInput(key, properties[key])}
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      {filteredKeys.length === 0 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center py-20"
        >
          <div className="w-20 h-20 rounded-full bg-black/5 dark:bg-white/[0.03] flex items-center justify-center mx-auto mb-6">
            <Search className="text-gray-400 dark:text-white/10" size={32} />
          </div>
          <h3 className="text-xl font-bold text-gray-400 dark:text-white/40">No properties found</h3>
          <p className="text-gray-500 dark:text-white/20 mt-2">Try searching for something else, like 'port' or 'pvp'</p>
        </motion.div>
      )}
    </div>
  )
}
