import { useState, useRef, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { ChevronDown, Check } from 'lucide-react'
import { cn } from '../utils'
import { useAppSettings } from '../hooks/useAppSettings'

interface Option {
  value: string
  label: string
}

interface SelectProps {
  value: string
  onChange: (value: string) => void
  options: Option[]
  className?: string
  placeholder?: string
  direction?: 'up' | 'down'
  disabled?: boolean
  loading?: boolean
  size?: 'sm' | 'md'
}

export function Select({
  value,
  onChange,
  options,
  className,
  placeholder,
  direction = 'down',
  disabled = false,
  loading = false,
  size = 'md'
}: SelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const triggerRef = useRef<HTMLButtonElement>(null)
  const dropdownRef = useRef<HTMLDivElement>(null)
  const [coords, setCoords] = useState({ top: 0, left: 0, width: 0 })
  const { settings } = useAppSettings()

  const selectedOption = options.find(opt => opt.value === value)

  const updatePosition = () => {
    if (triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect()
      setCoords({
        top: rect.top,
        left: rect.left,
        width: rect.width
      })
    }
  }

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current && !containerRef.current.contains(event.target as Node) &&
        dropdownRef.current && !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      updatePosition()
      document.addEventListener('mousedown', handleClickOutside)
      window.addEventListener('scroll', updatePosition, true)
      window.addEventListener('resize', updatePosition)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      window.removeEventListener('scroll', updatePosition, true)
      window.removeEventListener('resize', updatePosition)
    }
  }, [isOpen])

  const isUp = direction === 'up'
  const isDisabled = disabled || loading

  const dropdownContent = (
    <AnimatePresence>
      {isOpen && (
        <div
          style={{
            position: 'fixed',
            top: isUp ? `${coords.top - 8}px` : `${coords.top + (size === 'sm' ? 36 : 42)}px`,
            left: `${coords.left}px`,
            width: `${coords.width}px`,
            zIndex: 9999,
            transform: `scale(${settings.scaling}) ${isUp ? 'translateY(-100%)' : ''}`,
            transformOrigin: isUp ? 'bottom left' : 'top left',
          }}
        >
          <motion.div
            ref={dropdownRef}
            initial={{ opacity: 0, y: isUp ? -4 : 4, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: isUp ? -4 : 4, scale: 0.95 }}
            transition={{ duration: 0.15, ease: "easeOut" }}
            className={cn(
              "bg-white dark:bg-[#121214] border border-black/10 dark:border-white/10 rounded-xl shadow-2xl overflow-hidden backdrop-blur-xl ring-1 ring-black/5 dark:ring-white/5"
            )}
          >
            <div className="max-h-60 overflow-y-auto p-1.5 scrollbar-thin scrollbar-thumb-black/10 dark:scrollbar-thumb-white/10 scrollbar-track-transparent">
              {options.length === 0 ? (
                <div className="px-3 py-4 text-center text-xs text-gray-500 dark:text-white/30 font-medium italic">
                  No options available
                </div>
              ) : options.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => {
                    onChange(option.value)
                    setIsOpen(false)
                  }}
                  className={cn(
                    "w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-all group",
                    value === option.value
                      ? "bg-primary/20 text-primary font-bold"
                      : "text-gray-600 dark:text-white/60 hover:bg-black/5 dark:hover:bg-white/[0.05] hover:text-gray-900 dark:hover:text-white"
                  )}
                >
                  <span className="truncate">{option.label}</span>
                  {value === option.value && (
                    <Check size={14} className="shrink-0" />
                  )}
                </button>
              ))}
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )

  return (
    <div className={cn("relative w-full", className)} ref={containerRef}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => !isDisabled && setIsOpen(!isOpen)}
        disabled={isDisabled}
        className={cn(
          "w-full flex items-center justify-between gap-2 bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl text-sm text-gray-900 dark:text-white transition-all hover:bg-black/10 dark:hover:bg-white/[0.06] focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 text-left",
          size === 'sm' ? "px-3 py-1.5" : "px-4 py-2.5",
          isOpen && "border-primary/50 ring-2 ring-primary/50 bg-black/10 dark:bg-white/[0.08]",
          isDisabled && "opacity-50 cursor-not-allowed hover:bg-black/5 dark:hover:bg-white/[0.03]"
        )}
      >
        <div className="flex items-center gap-2 overflow-hidden">
          {loading && (
            <div className="w-3.5 h-3.5 border-2 border-primary/30 border-t-primary rounded-full animate-spin shrink-0" />
          )}
          <span className={cn("truncate font-medium", !selectedOption && "text-gray-500 dark:text-white/30")}>
            {loading ? 'Loading...' : (selectedOption ? selectedOption.label : placeholder)}
          </span>
        </div>
        <ChevronDown
          size={size === 'sm' ? 14 : 16}
          className={cn("text-gray-500 dark:text-white/30 transition-transform duration-200 shrink-0", isOpen && "rotate-180 text-primary")}
        />
      </button>

      {createPortal(dropdownContent, document.body)}
    </div>
  )
}
