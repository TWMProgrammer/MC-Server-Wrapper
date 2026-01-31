import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { FolderOpen, ExternalLink } from 'lucide-react'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'

interface InstanceFolderDropdownProps {
  instance: {
    id: string;
    name: string;
  };
}

export function InstanceFolderDropdown({ instance }: InstanceFolderDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  async function handleOpenFolder() {
    try {
      await invoke('open_instance_folder', { instanceId: instance.id });
      setIsOpen(false);
    } catch (e) {
      console.error('Failed to open folder', e);
      alert('Failed to open folder: ' + e);
    }
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <motion.button
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "transition-all p-2 rounded-xl border border-transparent",
          isOpen ? "text-primary bg-primary/10 border-primary/20" : "text-gray-400 dark:text-white/20 hover:text-gray-900 dark:hover:text-white/60 hover:bg-black/5 dark:hover:bg-white/5"
        )}
      >
        <FolderOpen size={18} />
      </motion.button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10, x: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0, x: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10, x: 20 }}
            className="absolute right-0 mt-3 w-64 bg-white dark:bg-gray-900 border border-black/10 dark:border-white/10 rounded-2xl shadow-2xl z-50 overflow-hidden ring-1 ring-black/5 dark:ring-white/10"
          >
            <div className="p-2">
              <button
                onClick={handleOpenFolder}
                className="w-full flex items-center gap-3 px-4 py-3 text-sm text-gray-700 dark:text-white/70 hover:bg-black/5 dark:hover:bg-white/[0.05] hover:text-gray-900 dark:hover:text-white rounded-xl transition-all group"
              >
                <div className="w-8 h-8 rounded-lg bg-black/[0.03] dark:bg-white/[0.03] flex items-center justify-center group-hover:bg-primary/20 group-hover:text-primary transition-all">
                  <ExternalLink size={16} />
                </div>
                <div className="flex flex-col items-start">
                  <span className="font-bold">Reveal in Explorer</span>
                  <span className="text-[10px] text-gray-400 dark:text-white/40 uppercase font-black tracking-widest">Open local files</span>
                </div>
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
