import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Settings } from 'lucide-react'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'
import { MainActions } from './instance-settings/MainActions'
import { CloneForm } from './instance-settings/CloneForm'
import { DeleteConfirm } from './instance-settings/DeleteConfirm'

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
                <MainActions
                  onShowClone={() => setShowCloneForm(true)}
                  onShowDelete={() => setShowDeleteConfirm(true)}
                />
              )}

              {showCloneForm && (
                <CloneForm
                  cloneName={cloneName}
                  setCloneName={setCloneName}
                  onClone={handleClone}
                  onBack={() => setShowCloneForm(false)}
                  isCloning={isCloning}
                />
              )}

              {showDeleteConfirm && (
                <DeleteConfirm
                  instanceName={instance.name}
                  onDelete={handleDelete}
                  onCancel={() => setShowDeleteConfirm(false)}
                  isDeleting={isDeleting}
                />
              )}
            </AnimatePresence>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
