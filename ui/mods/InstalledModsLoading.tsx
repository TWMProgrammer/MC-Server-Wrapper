import { RefreshCw } from 'lucide-react'

export function InstalledModsLoading() {
  return (
    <div className="flex flex-col items-center justify-center py-20 text-gray-500">
      <RefreshCw size={48} className="animate-spin mb-4 opacity-20" />
      <p className="font-medium animate-pulse">Loading mods...</p>
    </div>
  )
}
