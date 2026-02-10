import { Globe, HardDrive, Package } from 'lucide-react'
import { SidebarItem } from './SidebarItem'
import { Tab } from './types'

interface SidebarProps {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
}

export function Sidebar({ activeTab, setActiveTab }: SidebarProps) {
  return (
    <div className="w-60 bg-black/5 dark:bg-black/20 border-r border-black/5 dark:border-white/5 p-3 flex flex-col gap-1 transition-colors duration-300">
      <div className="px-3 py-1.5 text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Sources</div>
      <SidebarItem
        icon={<Globe size={18} />}
        label="Official Minecraft"
        active={activeTab === 'custom'}
        onClick={() => setActiveTab('custom')}
      />
      <SidebarItem
        icon={<HardDrive size={18} />}
        label="Import from Local"
        active={activeTab === 'import'}
        onClick={() => setActiveTab('import')}
      />
      <div className="my-3 border-t border-black/5 dark:border-white/5" />
      <div className="px-3 py-1.5 text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Modpacks</div>
      <SidebarItem
        icon={<Package size={18} />}
        label="Modrinth"
        active={activeTab === 'modrinth'}
        onClick={() => setActiveTab('modrinth')}
      />
      <SidebarItem
        icon={<Package size={18} />}
        label="CurseForge"
        active={activeTab === 'curseforge'}
        onClick={() => setActiveTab('curseforge')}
        disabled
      />
    </div>
  )
}
