import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, Check, X } from 'lucide-react';
import { cn } from '../utils';

interface ConfirmDropdownProps {
  onConfirm: () => void;
  title: string;
  message?: string;
  children: React.ReactNode;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'warning' | 'primary';
  className?: string;
  disabled?: boolean;
}

export function ConfirmDropdown({
  onConfirm,
  title,
  message,
  children,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  variant = 'primary',
  className,
  disabled
}: ConfirmDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [coords, setCoords] = useState({ top: 0, left: 0 });
  const containerRef = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const updatePosition = () => {
    if (containerRef.current) {
      const rect = containerRef.current.getBoundingClientRect();
      setCoords({
        top: rect.bottom + window.scrollY,
        left: rect.right + window.scrollX
      });
    }
  };

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current && !containerRef.current.contains(event.target as Node) &&
        dropdownRef.current && !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
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
  }, [isOpen]);

  const handleConfirm = () => {
    onConfirm();
    setIsOpen(false);
  };

  const variants = {
    danger: {
      button: "bg-accent-rose hover:bg-accent-rose/80 text-white shadow-glow-rose",
      icon: "text-accent-rose",
      bg: "bg-accent-rose/10 border-accent-rose/20",
    },
    warning: {
      button: "bg-accent-amber hover:bg-accent-amber/80 text-white shadow-glow-amber",
      icon: "text-accent-amber",
      bg: "bg-accent-amber/10 border-accent-amber/20",
    },
    primary: {
      button: "bg-primary hover:bg-primary/90 text-white shadow-glow-primary",
      icon: "text-primary",
      bg: "bg-primary/10 border-primary/20",
    }
  };

  const dropdownContent = (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          ref={dropdownRef}
          initial={{ opacity: 0, scale: 0.95, y: 10 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 10 }}
          style={{
            position: 'fixed',
            top: `${coords.top + 8}px`,
            left: `${coords.left - 256}px`, // 256px is w-64
            zIndex: 9999,
          }}
          className="w-64 bg-surface border border-white/10 rounded-2xl shadow-2xl overflow-hidden"
        >
          <div className="p-4 space-y-3">
            <div className="flex items-center gap-2">
              <div className={cn("p-1.5 rounded-lg", variants[variant].bg)}>
                <AlertTriangle size={16} className={variants[variant].icon} />
              </div>
              <h4 className="text-sm font-bold text-white uppercase tracking-tight">{title}</h4>
            </div>

            {message && (
              <p className="text-xs text-gray-400 leading-relaxed">
                {message}
              </p>
            )}

            <div className="flex flex-col gap-2 pt-1">
              <button
                onClick={handleConfirm}
                className={cn(
                  "w-full py-2 rounded-xl text-xs font-black uppercase tracking-widest transition-all flex items-center justify-center gap-2",
                  variants[variant].button
                )}
              >
                <Check size={14} />
                {confirmText}
              </button>
              <button
                onClick={() => setIsOpen(false)}
                className="w-full py-2 bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white rounded-xl text-[10px] font-black uppercase tracking-[0.2em] transition-all flex items-center justify-center gap-2"
              >
                <X size={14} />
                {cancelText}
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );

  return (
    <div className={cn("relative inline-block", className)} ref={containerRef}>
      <div onClick={() => !disabled && setIsOpen(!isOpen)}>
        {children}
      </div>
      {typeof document !== 'undefined' && createPortal(dropdownContent, document.body)}
    </div>
  );
}
