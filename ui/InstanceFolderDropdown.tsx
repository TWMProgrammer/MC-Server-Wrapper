import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { FolderOpen, ExternalLink } from 'lucide-react'
import { cn } from './utils'

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
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "hover:text-white transition-colors p-1 rounded-md",
          isOpen ? "text-white bg-white/10" : "text-gray-500"
        )}
      >
        <FolderOpen size={20} />
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-56 bg-[#242424] border border-white/10 rounded-lg shadow-xl z-50 overflow-hidden animate-in fade-in zoom-in-95 duration-100">
          <div className="p-1">
            <button
              onClick={handleOpenFolder}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:bg-white/5 hover:text-white rounded transition-colors"
            >
              <ExternalLink size={16} />
              <span>Open in File Explorer</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
