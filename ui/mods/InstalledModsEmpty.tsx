import { Layers } from 'lucide-react'

interface InstalledModsEmptyProps {
  searchQuery: string;
}

export function InstalledModsEmpty({ searchQuery }: InstalledModsEmptyProps) {
  return (
    <div className="flex flex-col items-center justify-center py-20 bg-white/5 rounded-3xl border border-dashed border-white/10">
      <div className="w-20 h-20 bg-white/5 rounded-full flex items-center justify-center mb-4">
        <Layers size={40} className="text-gray-600" />
      </div>
      <h3 className="text-xl font-bold text-gray-400">No mods found</h3>
      <p className="text-gray-500 max-w-xs text-center mt-2">
        {searchQuery ? `No mods matching "${searchQuery}"` : "This instance doesn't have any mods installed yet."}
      </p>
    </div>
  )
}
