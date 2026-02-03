import { Check } from 'lucide-react'
import { cn } from '../../utils'

export function Section({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) {
  return (
    <section className="space-y-4">
      <div className="flex items-center gap-2 px-1">
        <div className="p-1.5 bg-black/5 dark:bg-white/5 rounded-lg text-gray-500">
          <Icon size={16} />
        </div>
        <h3 className="text-sm font-bold text-gray-900 dark:text-white uppercase tracking-wider">{title}</h3>
      </div>
      <div className="space-y-4">
        {children}
      </div>
    </section>
  );
}

export function Checkbox({
  label,
  checked,
  onChange,
  description,
  disabled = false
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
  disabled?: boolean;
}) {
  return (
    <label className={cn(
      "flex items-start gap-3 p-3 rounded-xl transition-all cursor-pointer group",
      disabled ? "opacity-50 cursor-not-allowed" : "hover:bg-black/5 dark:hover:bg-white/5"
    )}>
      <div className="relative flex items-center mt-0.5">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => !disabled && onChange(e.target.checked)}
          className="peer sr-only"
          disabled={disabled}
        />
        <div className={cn(
          "w-5 h-5 rounded-md border-2 transition-all flex items-center justify-center",
          checked
            ? "bg-primary border-primary shadow-glow-primary/20"
            : "border-black/20 dark:border-white/20 group-hover:border-primary/50"
        )}>
          {checked && <Check size={14} className="text-white" />}
        </div>
      </div>
      <div className="flex-1">
        <div className={cn(
          "text-sm font-semibold transition-colors",
          checked ? "text-primary" : "text-gray-700 dark:text-gray-200"
        )}>
          {label}
        </div>
        {description && (
          <div className="text-xs text-gray-500 mt-0.5 leading-relaxed">
            {description}
          </div>
        )}
      </div>
    </label>
  )
}
