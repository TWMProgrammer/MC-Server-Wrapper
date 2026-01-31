import { X, Settings, Palette, Check, Moon, Sun } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '../utils'
import { ACCENT_COLORS, AccentColor, Theme } from '../hooks/useAppSettings'

interface AppSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  accentColor: AccentColor;
  onAccentColorChange: (color: AccentColor) => void;
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function AppSettingsModal({
  isOpen,
  onClose,
  accentColor,
  onAccentColorChange,
  theme,
  onThemeChange
}: AppSettingsModalProps) {
  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            className="relative w-full max-w-lg bg-surface border border-black/10 dark:border-white/10 rounded-2xl shadow-2xl overflow-hidden"
          >
            {/* Header */}
            <div className="p-6 border-b border-black/5 dark:border-white/5 flex items-center justify-between bg-black/[0.02] dark:bg-white/[0.02]">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-primary/20 rounded-lg">
                  <Settings className="text-primary w-5 h-5" />
                </div>
                <div>
                  <h2 className="text-xl font-bold text-gray-900 dark:text-white">App Settings</h2>
                  <p className="text-xs text-gray-500">Customize your workspace</p>
                </div>
              </div>
              <button
                onClick={onClose}
                className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg transition-colors text-gray-400 hover:text-gray-900 dark:hover:text-white"
              >
                <X size={20} />
              </button>
            </div>

            {/* Content */}
            <div className="p-6 space-y-8">
              {/* Theme Section */}
              <section>
                <div className="flex items-center gap-2 mb-4 text-sm font-semibold text-gray-400 uppercase tracking-wider">
                  <Sun size={16} />
                  <span>Theme</span>
                </div>
                <div className="flex gap-4 p-1 bg-black/10 dark:bg-white/5 rounded-xl">
                  <button
                    onClick={() => onThemeChange('light')}
                    className={cn(
                      "flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition-all duration-200",
                      theme === 'light'
                        ? "bg-white dark:bg-gray-800 text-primary shadow-sm"
                        : "text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                    )}
                  >
                    <Sun size={18} />
                    <span className="font-medium">Light</span>
                  </button>
                  <button
                    onClick={() => onThemeChange('dark')}
                    className={cn(
                      "flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition-all duration-200",
                      theme === 'dark'
                        ? "bg-white dark:bg-gray-800 text-primary shadow-sm"
                        : "text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
                    )}
                  >
                    <Moon size={18} />
                    <span className="font-medium">Dark</span>
                  </button>
                </div>
              </section>

              {/* Appearance Section */}
              <section>
                <div className="flex items-center gap-2 mb-4 text-sm font-semibold text-gray-400 uppercase tracking-wider">
                  <Palette size={16} />
                  <span>Accent Color</span>
                </div>

                <div className="space-y-4">
                  <div className="grid grid-cols-3 gap-3">
                    {ACCENT_COLORS.map((color) => (
                      <button
                        key={color.name}
                        onClick={() => onAccentColorChange(color)}
                        className={cn(
                          "flex items-center gap-3 px-4 py-3 rounded-xl border transition-all duration-200 group relative overflow-hidden",
                          accentColor.name === color.name
                            ? "bg-primary/10 border-primary shadow-glow-primary/20"
                            : "bg-white/[0.02] border-white/5 hover:border-white/20 hover:bg-white/[0.04]"
                        )}
                      >
                        <div
                          className="w-4 h-4 rounded-full shadow-sm shrink-0"
                          style={{ backgroundColor: `hsl(${color.value})` }}
                        />
                        <span className={cn(
                          "text-sm font-medium transition-colors",
                          accentColor.name === color.name ? "text-primary" : "text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-200"
                        )}>
                          {color.name}
                        </span>

                        {accentColor.name === color.name && (
                          <div className="ml-auto">
                            <Check size={16} className="text-primary" />
                          </div>
                        )}
                      </button>
                    ))}
                  </div>
                </div>
              </section>
            </div>

            {/* Footer */}
            <div className="p-6 bg-black/20 border-t border-white/5 flex justify-end">
              <button
                onClick={onClose}
                className="px-6 py-2 bg-primary text-white rounded-xl font-semibold shadow-glow-primary hover:shadow-primary/40 transition-all duration-200 active:scale-95"
              >
                Done
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}
