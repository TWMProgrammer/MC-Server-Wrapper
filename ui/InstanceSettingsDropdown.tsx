import { useState, useRef, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { invoke } from '@tauri-apps/api/core'
import { Settings } from 'lucide-react'
import { cn } from './utils'
import { motion, AnimatePresence } from 'framer-motion'
import { MainActions } from './instance-settings/MainActions'
import { CloneForm } from './instance-settings/CloneForm'
import { DeleteConfirm } from './instance-settings/DeleteConfirm'
import { useToast } from './hooks/useToast'
import { useAppSettings } from './hooks/useAppSettings'

interface InstanceSettingsDropdownProps {
  instance: {
    id: string;
    name: string;
  };
  onUpdated: (id?: string) => void;
  size?: number;
  className?: string;
  side?: 'left' | 'right';
}

export function InstanceSettingsDropdown({
  instance,
  onUpdated,
  size = 18,
  className,
  side = 'right'
}: InstanceSettingsDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showCloneForm, setShowCloneForm] = useState(false);
  const [cloneName, setCloneName] = useState(`${instance.name} (Copy)`);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isCloning, setIsCloning] = useState(false);
  const [coords, setCoords] = useState({ top: 0, left: 0 });
  const { showToast } = useToast();
  const { settings } = useAppSettings();

  const dropdownRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const updatePosition = () => {
    if (triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      setCoords({
        top: rect.bottom + window.scrollY,
        left: side === 'right' ? rect.right + window.scrollX : rect.left + window.scrollX
      });
    }
  };

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as Node;
      if (
        triggerRef.current && !triggerRef.current.contains(target) &&
        dropdownRef.current && !dropdownRef.current.contains(target)
      ) {
        setIsOpen(false);
        setTimeout(() => {
          setShowDeleteConfirm(false);
          setShowCloneForm(false);
        }, 200);
      }
    }

    if (isOpen) {
      updatePosition();
      document.addEventListener('mousedown', handleClickOutside);
      window.addEventListener('scroll', updatePosition, true);
      window.addEventListener('resize', updatePosition);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      window.removeEventListener('scroll', updatePosition, true);
      window.removeEventListener('resize', updatePosition);
    };
  }, [isOpen, side]);

  async function handleDelete() {
    try {
      setIsDeleting(true);
      await invoke('delete_instance', { instanceId: instance.id });
      onUpdated();
      setIsOpen(false);
      showToast('Instance deleted successfully');
    } catch (e) {
      console.error('Failed to delete instance', e);
      showToast('Failed to delete instance: ' + e, 'error');
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
      showToast('Instance cloned successfully');
    } catch (e) {
      console.error('Failed to clone instance', e);
      showToast('Failed to clone instance: ' + e, 'error');
    } finally {
      setIsCloning(false);
    }
  }

  const dropdownContent = (
    <AnimatePresence mode="wait">
      {isOpen && (
        <div
          style={{
            position: 'fixed',
            top: `${coords.top + 12}px`,
            left: side === 'right' ? `${coords.left - 288}px` : `${coords.left}px`, // 288px is w-72
            zIndex: 9999,
            transform: `scale(${settings.scaling})`,
            transformOrigin: side === 'right' ? 'top right' : 'top left',
          }}
        >
          <motion.div
            ref={dropdownRef}
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            className="w-72 bg-white dark:bg-gray-900 border border-black/10 dark:border-white/10 rounded-2xl shadow-2xl overflow-hidden ring-1 ring-black/5 dark:ring-white/10"
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
        </div>
      )}
    </AnimatePresence>
  );

  return (
    <>
      <motion.button
        ref={triggerRef}
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={(e) => {
          e.stopPropagation();
          setIsOpen(!isOpen);
        }}
        className={cn(
          "transition-all p-2 rounded-xl border border-transparent",
          isOpen ? "text-primary bg-primary/10 border-primary/20" : "text-gray-400 dark:text-white/20 hover:text-gray-900 dark:hover:text-white/60 hover:bg-black/5 dark:hover:bg-white/5",
          className
        )}
      >
        <Settings size={size} />
      </motion.button>
      {createPortal(dropdownContent, document.body)}
    </>
  );
}
