import { motion, AnimatePresence } from 'framer-motion'
import { Settings2, Search } from 'lucide-react'
import { Select } from '../components/Select'
import { YamlTreeEditor } from './YamlTreeEditor'
import { ConfigFile } from './types'

interface PropertyGridProps {
  selectedConfig: ConfigFile | null
  nestedConfig: any
  setNestedConfig: (val: any) => void
  filteredKeys: string[]
  properties: Record<string, string>
  onPropertyChange: (key: string, value: string) => void
}

export function PropertyGrid({
  selectedConfig,
  nestedConfig,
  setNestedConfig,
  filteredKeys,
  properties,
  onPropertyChange
}: PropertyGridProps) {
  const renderInput = (key: string, value: string) => {
    const isBoolean = value === 'true' || value === 'false'
    const isNumeric = !isNaN(Number(value)) && value.trim() !== ''

    const inputClasses = "w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all hover:bg-black/10 dark:hover:bg-white/[0.05]"

    if (isBoolean) {
      return (
        <Select
          value={value}
          onChange={(newValue) => onPropertyChange(key, newValue)}
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
          onChange={(e) => onPropertyChange(key, e.target.value)}
        />
      )
    }

    return (
      <input
        type="text"
        className={inputClasses}
        value={value}
        onChange={(e) => onPropertyChange(key, e.target.value)}
      />
    )
  }

  return (
    <div className="flex-1 space-y-8 pb-8">
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-2 2xl:grid-cols-3 gap-4">
        <AnimatePresence mode="popLayout">
          {selectedConfig?.name === 'commands.yml' && nestedConfig ? (
            <>
              <motion.div
                layout
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                className="glass-panel p-5 rounded-2xl border border-black/5 dark:border-white/5 flex flex-col gap-3 group hover:border-primary/30 transition-all duration-200 hover:translate-y-[-2px] focus-within:z-20 relative"
              >
                <div className="flex items-center justify-between">
                  <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors truncate pr-4">
                    Command Block Overrides
                  </label>
                  <Settings2 size={14} className="text-gray-400 dark:text-white/10 group-hover:text-primary transition-colors shrink-0" />
                </div>
                <YamlTreeEditor
                  value={nestedConfig['command-block-overrides'] ?? []}
                  allowedTypes={['string']}
                  typeLabels={{ string: 'command' }}
                  onChange={(newVal) => setNestedConfig({ ...nestedConfig, 'command-block-overrides': newVal })}
                />
              </motion.div>

              <motion.div
                layout
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                className="glass-panel p-5 rounded-2xl border border-black/5 dark:border-white/5 flex flex-col gap-3 group hover:border-primary/30 transition-all duration-200 hover:translate-y-[-2px] focus-within:z-20 relative"
              >
                <div className="flex items-center justify-between">
                  <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors truncate pr-4">
                    Ignore Vanilla Permissions
                  </label>
                  <Settings2 size={14} className="text-gray-400 dark:text-white/10 group-hover:text-primary transition-colors shrink-0" />
                </div>
                <Select
                  value={String(nestedConfig['ignore-vanilla-permissions'] ?? false)}
                  onChange={(newVal) => setNestedConfig({ ...nestedConfig, 'ignore-vanilla-permissions': newVal === 'true' })}
                  options={[
                    { value: 'true', label: 'true' },
                    { value: 'false', label: 'false' },
                  ]}
                />
              </motion.div>

              <div className="col-span-full py-4 flex items-center gap-4">
                <div className="h-px flex-1 bg-black/5 dark:bg-white/5" />
                <span className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20">Advanced Configuration</span>
                <div className="h-px flex-1 bg-black/5 dark:bg-white/5" />
              </div>

              {Object.entries(nestedConfig)
                .filter(([k]) => k !== 'command-block-overrides' && k !== 'ignore-vanilla-permissions')
                .map(([k, v]) => (
                  <div key={k} className="col-span-full">
                    <YamlTreeEditor
                      label={k}
                      value={v}
                      allowedTypes={k === 'aliases' ? ['array'] : undefined}
                      typeLabels={k === 'aliases' ? { array: 'alias', string: 'command' } : undefined}
                      onChange={(newVal) => setNestedConfig({ ...nestedConfig, [k]: newVal })}
                    />
                  </div>
                ))}
            </>
          ) : (
            filteredKeys.map((key, index) => (
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
                  <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors truncate pr-4">
                    {key.replace(/-/g, ' ')}
                  </label>
                  <Settings2 size={14} className="text-gray-400 dark:text-white/10 group-hover:text-primary transition-colors shrink-0" />
                </div>
                {renderInput(key, properties[key])}
              </motion.div>
            ))
          )}
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
