import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Trash2, Copy, AlertTriangle, Settings, ChevronRight } from 'lucide-react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface InstanceSettingsDropdownProps {
  instance: {
    id: string;
    name: string;
  };
  onUpdated: () => void;
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
        setShowDeleteConfirm(false);
        setShowCloneForm(false);
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
      await invoke('clone_instance', { instanceId: instance.id, newName: cloneName });
      onUpdated();
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
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "hover:text-white transition-colors p-1 rounded-md",
          isOpen ? "text-white bg-white/10" : "text-gray-500"
        )}
      >
        <Settings size={20} />
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-64 bg-[#242424] border border-white/10 rounded-lg shadow-xl z-50 overflow-hidden animate-in fade-in zoom-in-95 duration-100">
          {!showDeleteConfirm && !showCloneForm && (
            <div className="p-1">
              <button
                onClick={() => setShowCloneForm(true)}
                className="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:bg-white/5 hover:text-white rounded transition-colors"
              >
                <Copy size={16} />
                <span>Clone Instance</span>
              </button>
              <button
                onClick={() => setShowDeleteConfirm(true)}
                className="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-red-500/10 rounded transition-colors"
              >
                <Trash2 size={16} />
                <span>Delete Instance</span>
              </button>
            </div>
          )}

          {showCloneForm && (
            <div className="p-3 space-y-3">
              <div className="text-xs font-semibold text-gray-500 uppercase">Clone Instance</div>
              <input
                type="text"
                value={cloneName}
                onChange={e => setCloneName(e.target.value)}
                className="w-full bg-[#1a1a1a] border border-white/10 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-green-500"
                placeholder="New name..."
                autoFocus
              />
              <div className="flex gap-2">
                <button
                  onClick={() => setShowCloneForm(false)}
                  className="flex-1 px-2 py-1.5 text-xs bg-white/5 hover:bg-white/10 text-white rounded transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleClone}
                  disabled={isCloning || !cloneName.trim()}
                  className="flex-1 px-2 py-1.5 text-xs bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded font-bold transition-colors"
                >
                  {isCloning ? 'Cloning...' : 'Clone'}
                </button>
              </div>
            </div>
          )}

          {showDeleteConfirm && (
            <div className="p-3 space-y-3">
              <div className="flex items-center gap-2 text-red-400">
                <AlertTriangle size={16} />
                <span className="text-xs font-bold uppercase">Are you sure?</span>
              </div>
              <p className="text-[11px] text-gray-400">
                This will permanently delete all files for <strong>{instance.name}</strong>.
              </p>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowDeleteConfirm(false)}
                  className="flex-1 px-2 py-1.5 text-xs bg-white/5 hover:bg-white/10 text-white rounded transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleDelete}
                  disabled={isDeleting}
                  className="flex-1 px-2 py-1.5 text-xs bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white rounded font-bold transition-colors"
                >
                  {isDeleting ? 'Delete' : 'Delete'}
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
