import { Globe, HardDrive, Package } from 'lucide-react'
import { SidebarItem } from './SidebarItem'
import { Tab } from './types'

interface SidebarProps {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
}

export function Sidebar({ activeTab, setActiveTab }: SidebarProps) {
  return (
    <div className="w-72 bg-black/5 dark:bg-black/20 border-r border-black/5 dark:border-white/5 p-4 flex flex-col gap-2 transition-colors duration-300">
      <div className="px-4 py-2 text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Sources</div>
      <SidebarItem
        icon={<Globe size={20} />}
        label="Official Minecraft"
        active={activeTab === 'custom'}
        onClick={() => setActiveTab('custom')}
      />
      <SidebarItem
        icon={<HardDrive size={20} />}
        label="Local ZIP File"
        active={activeTab === 'import'}
        onClick={() => setActiveTab('import')}
      />
      <div className="my-4 border-t border-black/5 dark:border-white/5" />
      <div className="px-4 py-2 text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/30">Coming Soon</div>
      <SidebarItem
        icon={<Package size={20} />}
        label="Modrinth"
        active={activeTab === 'modrinth'}
        onClick={() => setActiveTab('modrinth')}
        disabled
      />
      <SidebarItem
        icon={<Package size={20} />}
        label="CurseForge"
        active={activeTab === 'curseforge'}
        onClick={() => setActiveTab('curseforge')}
        disabled
      />
    </div>
  )
}
