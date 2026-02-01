import { useState } from 'react'
import { Plus, Trash2, MoreVertical } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'

interface YamlTreeEditorProps {
  value: any
  onChange: (val: any) => void
  label?: string
  allowedTypes?: ('string' | 'object' | 'array')[]
  typeLabels?: Partial<Record<'string' | 'object' | 'array', string>>
}

export function YamlTreeEditor({ value, onChange, label, allowedTypes, typeLabels }: YamlTreeEditorProps) {
  const [isHovered, setIsHovered] = useState(false)

  const isObject = value !== null && typeof value === 'object' && !Array.isArray(value)
  const isArray = Array.isArray(value)

  const handleAdd = (type: 'string' | 'object' | 'array') => {
    let newValue
    if (type === 'string') newValue = ""
    else if (type === 'object') newValue = {}
    else if (type === 'array') newValue = []

    if (isArray) {
      onChange([...value, newValue])
    } else if (isObject) {
      const key = prompt("Enter key name:")
      if (key) {
        onChange({ ...value, [key]: newValue })
      }
    }
  }

  const renderActionButtons = () => {
    const types = allowedTypes || ['string', 'object', 'array']

    return (
      <div className="flex items-center gap-1">
        {types.map((type) => (
          <button
            key={type}
            onClick={() => handleAdd(type)}
            className="px-2 py-1 rounded-md bg-primary/10 hover:bg-primary/20 text-primary text-[10px] font-bold uppercase tracking-wider flex items-center gap-1 transition-all"
          >
            <Plus size={10} /> {typeLabels?.[type] || type}
          </button>
        ))}
      </div>
    )
  }

  if (isObject) {
    return (
      <div
        className="glass-panel p-5 rounded-2xl border border-black/5 dark:border-white/5 flex flex-col gap-4 group hover:border-primary/30 transition-all duration-200 relative"
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <div className="flex items-center justify-between">
          <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors">
            {label || 'Object'}
          </label>
          {renderActionButtons()}
        </div>

        <div className="space-y-4">
          {Object.entries(value).map(([k, v]) => (
            <div key={k} className="space-y-2">
              <div className="flex items-center justify-between group/item">
                <span className="text-[10px] font-bold uppercase tracking-wider text-gray-400 dark:text-white/20">{k}</span>
                <button
                  onClick={() => {
                    const newValue = { ...value }
                    delete newValue[k]
                    onChange(newValue)
                  }}
                  className="opacity-0 group-hover/item:opacity-100 p-1 hover:text-rose-500 transition-all"
                >
                  <Trash2 size={12} />
                </button>
              </div>
              <YamlTreeEditor
                value={v}
                allowedTypes={Array.isArray(v) ? ['string'] : allowedTypes}
                typeLabels={typeLabels}
                onChange={(newVal) => onChange({ ...value, [k]: newVal })}
              />
            </div>
          ))}
          {Object.keys(value).length === 0 && (
            <div className="text-center py-4 border-2 border-dashed border-black/5 dark:border-white/5 rounded-xl">
              <span className="text-xs text-gray-400 dark:text-white/10">Empty Object</span>
            </div>
          )}
        </div>
      </div>
    )
  }

  if (isArray) {
    return (
      <div
        className="glass-panel p-5 rounded-2xl border border-black/5 dark:border-white/5 flex flex-col gap-4 group hover:border-primary/30 transition-all duration-200 relative"
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        <div className="flex items-center justify-between">
          <label className="text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/30 group-hover:text-primary/70 transition-colors">
            {label || 'Array'}
          </label>
          {renderActionButtons()}
        </div>

        <div className="space-y-3">
          {value.map((item: any, i: number) => (
            <div key={i} className="flex items-center gap-3 group/item">
              <div className="flex-1">
                <YamlTreeEditor
                  value={item}
                  allowedTypes={allowedTypes}
                  typeLabels={typeLabels}
                  onChange={(newVal) => {
                    const newValue = [...value]
                    newValue[i] = newVal
                    onChange(newValue)
                  }}
                />
              </div>
              <button
                onClick={() => {
                  const newValue = value.filter((_: any, index: number) => index !== i)
                  onChange(newValue)
                }}
                className="opacity-0 group-hover/item:opacity-100 p-2 hover:text-rose-500 transition-all shrink-0"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
          {value.length === 0 && (
            <div className="text-center py-4 border-2 border-dashed border-black/5 dark:border-white/5 rounded-xl">
              <span className="text-xs text-gray-400 dark:text-white/10">Empty Array</span>
            </div>
          )}
        </div>
      </div>
    )
  }

  // Primitive value
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all hover:bg-black/10 dark:hover:bg-white/[0.05]"
    />
  )
}
