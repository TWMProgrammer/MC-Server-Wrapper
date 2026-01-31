import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Trash2, Copy, AlertTriangle, Settings, ChevronLeft, RefreshCw, X } from 'lucide-react'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'

interface InstanceSettingsDropdownProps {
  instance: {
    id: string;
    name: string;
  };
  onUpdated: (id?: string) => void;
}

export function InstanceSettingsDropdown({ instance, onUpdated }: InstanceSettingsDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showCloneForm, setShowCloneForm] = useState(false);
  const [cloneName, setCloneName] = useState(`${instance.name} (Copy)`);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isCloning, setIsCloning] = useState(false);

  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setTimeout(() => {
          setShowDeleteConfirm(false);
          setShowCloneForm(false);
        }, 200);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  async function handleDelete() {
    try {
      setIsDeleting(true);
      await invoke('delete_instance', { instanceId: instance.id });
      onUpdated();
      setIsOpen(false);
    } catch (e) {
      console.error('Failed to delete instance', e);
      alert('Failed to delete instance: ' + e);
    } finally {
      setIsDeleting(false);
    }
  }

  async function handleClone() {
    if (!cloneName.trim()) return;
    try {
      setIsCloning(true);
      const newInstance = await invoke<{ id: string }>('clone_instance', { instanceId: instance.id, newName: cloneName });
      onUpdated(newInstance.id);
      setIsOpen(false);
    } catch (e) {
      console.error('Failed to clone instance', e);
      alert('Failed to clone instance: ' + e);
    } finally {
      setIsCloning(false);
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
        <Settings size={18} />
      </motion.button>

      <AnimatePresence mode="wait">
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10, x: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0, x: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10, x: 20 }}
            className="absolute right-0 mt-3 w-72 bg-white dark:bg-gray-900 border border-black/10 dark:border-white/10 rounded-2xl shadow-2xl z-50 overflow-hidden ring-1 ring-black/5 dark:ring-white/10"
          >
            <AnimatePresence mode="wait">
              {!showDeleteConfirm && !showCloneForm && (
                <motion.div
                  key="main"
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  className="p-2 space-y-1"
                >
                  <div className="px-3 py-2 mb-1">
                    <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-500 dark:text-white/50">Instance Actions</h4>
                  </div>
                  <button
                    onClick={() => setShowCloneForm(true)}
                    className="w-full flex items-center gap-3 px-3 py-3 text-sm text-gray-700 dark:text-white/70 hover:bg-black/5 dark:hover:bg-white/[0.05] hover:text-gray-900 dark:hover:text-white rounded-xl transition-all group"
                  >
                    <div className="w-8 h-8 rounded-lg bg-black/[0.03] dark:bg-white/[0.03] flex items-center justify-center group-hover:bg-primary/20 group-hover:text-primary transition-all">
                      <Copy size={16} />
                    </div>
                    <div className="flex flex-col items-start">
                      <span className="font-bold">Clone Instance</span>
                      <span className="text-[10px] text-gray-400 dark:text-white/40 uppercase font-black tracking-widest">Duplicate this server</span>
                    </div>
                  </button>
                  <button
                    onClick={() => setShowDeleteConfirm(true)}
                    className="w-full flex items-center gap-3 px-3 py-3 text-sm text-gray-700 dark:text-white/70 hover:bg-accent-rose/10 hover:text-accent-rose rounded-xl transition-all group"
                  >
                    <div className="w-8 h-8 rounded-lg bg-black/[0.03] dark:bg-white/[0.03] flex items-center justify-center group-hover:bg-accent-rose/20 group-hover:text-accent-rose transition-all">
                      <Trash2 size={16} />
                    </div>
                    <div className="flex flex-col items-start">
                      <span className="font-bold">Delete Instance</span>
                      <span className="text-[10px] text-gray-400 dark:text-white/40 uppercase font-black tracking-widest">Permanent removal</span>
                    </div>
                  </button>
                </motion.div>
              )}

              {showCloneForm && (
                <motion.div
                  key="clone"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  className="p-4 space-y-4"
                >
                  <div className="flex items-center gap-3 mb-2">
                    <button
                      onClick={() => setShowCloneForm(false)}
                      className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-colors"
                    >
                      <ChevronLeft size={16} />
                    </button>
                    <span className="text-xs font-black uppercase tracking-widest text-gray-500 dark:text-white/60">Clone Server</span>
                  </div>

                  <div className="space-y-2">
                    <label className="text-[10px] font-black uppercase tracking-widest text-gray-400 dark:text-white/20 ml-1">New Instance Name</label>
                    <input
                      type="text"
                      value={cloneName}
                      onChange={e => setCloneName(e.target.value)}
                      className="w-full bg-black/5 dark:bg-white/[0.03] border border-black/10 dark:border-white/10 rounded-xl px-4 py-3 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50 transition-all"
                      placeholder="New name..."
                      autoFocus
                    />
                  </div>

                  <div className="flex gap-2 pt-2">
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      onClick={handleClone}
                      disabled={isCloning || !cloneName.trim()}
                      className="flex-1 py-3 bg-primary hover:bg-primary-hover disabled:opacity-50 text-white rounded-xl text-xs font-black uppercase tracking-widest shadow-glow-primary transition-all flex items-center justify-center gap-2"
                    >
                      {isCloning ? <RefreshCw size={14} className="animate-spin" /> : <Copy size={14} />}
                      {isCloning ? 'Cloning...' : 'Clone'}
                    </motion.button>
                  </div>
                </motion.div>
              )}

              {showDeleteConfirm && (
                <motion.div
                  key="delete"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  className="p-4 space-y-4"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2 text-accent-rose">
                      <AlertTriangle size={18} />
                      <span className="text-xs font-black uppercase tracking-widest">Dangerous</span>
                    </div>
                    <button
                      onClick={() => setShowDeleteConfirm(false)}
                      className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white transition-colors"
                    >
                      <X size={16} />
                    </button>
                  </div>

                  <div className="space-y-2">
                    <p className="text-xs text-gray-600 dark:text-white/60 leading-relaxed">
                      Are you sure you want to delete <strong className="text-gray-900 dark:text-white font-bold">{instance.name}</strong>? This action cannot be undone.
                    </p>
                  </div>

                  <div className="flex flex-col gap-2 pt-2">
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      onClick={handleDelete}
                      disabled={isDeleting}
                      className="w-full py-3 bg-accent-rose hover:bg-accent-rose/80 disabled:opacity-50 text-white rounded-xl text-xs font-black uppercase tracking-widest shadow-glow-rose transition-all"
                    >
                      {isDeleting ? 'Deleting...' : 'Permanently Delete'}
                    </motion.button>
                    <button
                      onClick={() => setShowDeleteConfirm(false)}
                      className="w-full py-3 bg-black/5 dark:bg-white/[0.03] hover:bg-black/10 dark:hover:bg-white/[0.08] text-gray-500 dark:text-white/60 hover:text-gray-900 dark:hover:text-white rounded-xl text-[10px] font-black uppercase tracking-[0.2em] transition-all"
                    >
                      Keep Instance
                    </button>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
