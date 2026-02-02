import { ChevronRight } from 'lucide-react'
import { cn } from '../utils'

interface SidebarItemProps {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
  disabled?: boolean;
}

export function SidebarItem({ icon, label, active, onClick, disabled = false }: SidebarItemProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200",
        active
          ? "bg-primary text-white shadow-lg shadow-primary/20"
          : disabled
            ? "text-gray-400 dark:text-white/10 cursor-not-allowed"
            : "text-gray-500 dark:text-white/50 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5"
      )}
    >
      {icon}
      {label}
      {active && <ChevronRight size={14} className="ml-auto opacity-50" />}
    </button>
  )
}
