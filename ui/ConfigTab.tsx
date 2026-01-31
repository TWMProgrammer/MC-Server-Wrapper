import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Save, RefreshCw, Search } from 'lucide-react'

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
      <div className="flex items-center justify-center h-full">
        <RefreshCw className="animate-spin text-blue-500 mr-2" />
        <span>Loading properties...</span>
      </div>
    )
  }

  const renderInput = (key: string, value: string) => {
    const isBoolean = value === 'true' || value === 'false'
    const isNumeric = !isNaN(Number(value)) && value.trim() !== ''

    if (isBoolean) {
      return (
        <select
          className="bg-[#1e1e1e] border border-[#333] rounded px-3 py-1.5 focus:outline-none focus:border-blue-500 w-full"
          value={value}
          onChange={(e) => handlePropertyChange(key, e.target.value)}
        >
          <option value="true">true</option>
          <option value="false">false</option>
        </select>
      )
    }

    if (isNumeric) {
      return (
        <input
          type="number"
          className="bg-[#1e1e1e] border border-[#333] rounded px-3 py-1.5 focus:outline-none focus:border-blue-500 w-full"
          value={value}
          onChange={(e) => handlePropertyChange(key, e.target.value)}
        />
      )
    }

    return (
      <input
        type="text"
        className="bg-[#1e1e1e] border border-[#333] rounded px-3 py-1.5 focus:outline-none focus:border-blue-500 w-full"
        value={value}
        onChange={(e) => handlePropertyChange(key, e.target.value)}
      />
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
          <input
            type="text"
            placeholder="Search properties..."
            className="w-full bg-[#2a2a2a] border border-[#333] rounded-lg py-2 pl-10 pr-4 focus:outline-none focus:ring-2 focus:ring-blue-500"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <div className="flex gap-3">
          <button
            onClick={fetchProperties}
            className="flex items-center gap-2 px-4 py-2 bg-[#2a2a2a] hover:bg-[#333] rounded-lg transition-colors"
          >
            <RefreshCw size={18} />
            Refresh
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-800 disabled:opacity-50 rounded-lg transition-colors"
          >
            <Save size={18} />
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/50 text-red-500 p-4 rounded-lg">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {filteredKeys.map(key => (
          <div key={key} className="bg-[#2a2a2a] p-4 rounded-lg border border-[#333] flex flex-col gap-2">
            <label className="text-sm font-medium text-gray-400">{key}</label>
            {renderInput(key, properties[key])}
          </div>
        ))}
      </div>

      {filteredKeys.length === 0 && (
        <div className="text-center py-10 text-gray-500">
          No properties found matching your search.
        </div>
      )}
    </div>
  )
}
