import { useState, useEffect, useRef } from 'react'
import { Save, X, Maximize2, Minimize2, Share } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '../utils'

interface TextEditorProps {
  initialValue: string
  onSave: (value: string) => Promise<void>
  onClose: () => void
  onOpenExternal?: () => void
  title: string
  language?: string
}

export function TextEditor({ initialValue, onSave, onClose, onOpenExternal, title }: TextEditorProps) {
  const [value, setValue] = useState(initialValue)
  const [isSaving, setIsSaving] = useState(false)
  const [isMaximized, setIsMaximized] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const lineNumbersRef = useRef<HTMLDivElement>(null)

  const lineCount = value.split('\n').length

  const handleScroll = () => {
    if (textareaRef.current && lineNumbersRef.current) {
      lineNumbersRef.current.scrollTop = textareaRef.current.scrollTop
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Tab') {
      e.preventDefault()
      const start = e.currentTarget.selectionStart
      const end = e.currentTarget.selectionEnd
      const newValue = value.substring(0, start) + '    ' + value.substring(end)
      setValue(newValue)

      // Reset selection after state update
      setTimeout(() => {
        if (textareaRef.current) {
          textareaRef.current.selectionStart = textareaRef.current.selectionEnd = start + 4
        }
      }, 0)
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      handleSave()
    }
  }

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await onSave(value)
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      className={cn(
        "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm",
        isMaximized && "p-0"
      )}
    >
      <div className={cn(
        "bg-[#1e1e1e] border border-white/10 shadow-2xl flex flex-col transition-all duration-300 overflow-hidden",
        isMaximized ? "w-full h-full" : "w-full max-w-5xl h-[80vh] rounded-2xl"
      )}>
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-white/[0.02]">
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full bg-red-500/20 border border-red-500/50" />
            <div className="w-3 h-3 rounded-full bg-amber-500/20 border border-amber-500/50" />
            <div className="w-3 h-3 rounded-full bg-emerald-500/20 border border-emerald-500/50" />
            <span className="ml-4 text-sm font-medium text-white/60 tracking-wider uppercase">{title}</span>
          </div>
          <div className="flex items-center gap-2">
            {onOpenExternal && (
              <button
                onClick={onOpenExternal}
                className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-white transition-all group relative"
                title="Open in default editor"
              >
                <Share size={18} />
                <span className="absolute -bottom-10 left-1/2 -translate-x-1/2 px-2 py-1 bg-black text-white text-[10px] rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none border border-white/10">
                  Open in External Editor
                </span>
              </button>
            )}
            <button
              onClick={() => setIsMaximized(!isMaximized)}
              className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-white transition-all"
            >
              {isMaximized ? <Minimize2 size={18} /> : <Maximize2 size={18} />}
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg text-sm font-medium hover:brightness-110 disabled:opacity-50 transition-all shadow-glow-primary"
            >
              <Save size={16} />
              {isSaving ? 'Saving...' : 'Save'}
            </button>
            <button
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-white/5 text-white/40 hover:text-white transition-all"
            >
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Editor Area */}
        <div className="flex-1 flex overflow-hidden relative font-mono text-sm leading-relaxed">
          {/* Line Numbers */}
          <div
            ref={lineNumbersRef}
            className="w-12 shrink-0 bg-[#1e1e1e] border-r border-white/5 py-4 text-right pr-3 text-white/20 select-none overflow-hidden"
          >
            {Array.from({ length: lineCount }).map((_, i) => (
              <div key={i} className="h-[1.625rem]">{i + 1}</div>
            ))}
          </div>

          {/* Textarea */}
          <textarea
            ref={textareaRef}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onScroll={handleScroll}
            onKeyDown={handleKeyDown}
            spellCheck={false}
            className="flex-1 bg-transparent text-white/80 p-4 resize-none focus:outline-none overflow-auto custom-scrollbar selection:bg-primary/30"
            placeholder="Start typing..."
          />
        </div>

        {/* Footer */}
        <div className="px-6 py-2 border-t border-white/5 bg-white/[0.01] flex items-center justify-between text-[10px] text-white/20 font-medium uppercase tracking-[0.2em]">
          <span>Lines: {lineCount}</span>
          <span>UTF-8</span>
        </div>
      </div>
    </motion.div>
  )
}
