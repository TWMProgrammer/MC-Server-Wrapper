import { Info, Plus, Power } from 'lucide-react'
import { motion } from 'framer-motion'
import { cn } from '../utils'
import { Tab } from './types'
import { ConfirmDropdown } from '../components/ConfirmDropdown'

interface FooterProps {
  selectedVersion: string | null;
  name: string;
  creating: boolean;
  loadingModLoaders: boolean;
  onClose: () => void;
  onCreate: () => void;
  activeTab: Tab;
  importSourcePath: string | null;
  selectedJar: string | null;
  serverPropertiesExists: boolean;
  startAfterCreation: boolean;
  setStartAfterCreation: (val: boolean) => void;
}

export function Footer({
  selectedVersion,
  name,
  creating,
  loadingModLoaders,
  onClose,
  onCreate,
  activeTab,
  importSourcePath,
  selectedJar,
  serverPropertiesExists,
  startAfterCreation,
  setStartAfterCreation
}: FooterProps) {
  const isImport = activeTab === 'import';
  const isDisabled = isImport
    ? !name || !importSourcePath || !selectedJar || creating
    : !name || !selectedVersion || creating || loadingModLoaders;

  const showWarning = isImport && !serverPropertiesExists && !isDisabled;

  const getReadyMessage = () => {
    if (isImport) {
      if (name && importSourcePath && selectedJar) return `Ready to import ${name}`;
      return 'Select a source and JAR to continue';
    }
    return selectedVersion ? `Ready to install Minecraft ${selectedVersion}` : 'Select a software and version to continue';
  };

  const createButton = (
    <motion.button
      whileHover={isDisabled ? {} : { scale: 1.02, translateY: -2 }}
      whileTap={isDisabled ? {} : { scale: 0.98 }}
      onClick={showWarning ? undefined : onCreate}
      disabled={isDisabled}
      className={cn(
        "px-8 py-2.5 rounded-xl text-xs font-black uppercase tracking-widest transition-all duration-200 flex items-center gap-3 shadow-2xl",
        isDisabled
          ? "bg-black/5 dark:bg-white/5 text-gray-400 dark:text-white/10 cursor-not-allowed"
          : "bg-primary hover:bg-primary-hover text-white shadow-glow-primary"
      )}
    >
      {creating ? (
        <><div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin" /> Creating...</>
      ) : (
        <><Plus size={18} /> {isImport ? 'Import Instance' : 'Create Instance'}</>
      )}
    </motion.button>
  );

  return (
    <div className="p-4 border-t border-black/5 dark:border-white/5 flex items-center justify-between bg-black/5 dark:bg-black/40 backdrop-blur-xl transition-colors duration-300">
      <div className="text-[9px] font-black uppercase tracking-[0.2em] text-gray-400 dark:text-white/20 flex items-center gap-2.5">
        <Info size={14} className="text-primary" />
        <span>{getReadyMessage()}</span>
      </div>
      <div className="flex items-center gap-5">
        {!isImport && (
          <button
            onClick={() => setStartAfterCreation(!startAfterCreation)}
            className={cn(
              "flex items-center gap-2.5 px-3 py-1.5 rounded-xl transition-all duration-300 group",
              startAfterCreation
                ? "bg-primary/10 text-primary border border-primary/20"
                : "bg-black/5 dark:bg-white/5 text-gray-400 border border-transparent"
            )}
          >
            <div className={cn(
              "w-7 h-3.5 rounded-full relative transition-colors duration-300",
              startAfterCreation ? "bg-primary" : "bg-gray-300 dark:bg-white/10"
            )}>
              <motion.div
                animate={{ x: startAfterCreation ? 16 : 2 }}
                className="absolute top-0.75 w-2 h-2 rounded-full bg-white shadow-sm"
              />
            </div>
            <div className="flex flex-col items-start">
              <span className="text-[9px] font-black uppercase tracking-widest leading-none">Start Server</span>
              <span className="text-[7px] font-bold opacity-50 uppercase tracking-tighter">After Creation</span>
            </div>
          </button>
        )}

        <button
          onClick={onClose}
          className="px-6 py-2.5 rounded-xl text-xs font-black uppercase tracking-widest text-gray-400 dark:text-white/40 hover:text-gray-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-all"
        >
          Cancel
        </button>

        {showWarning ? (
          <ConfirmDropdown
            title="Standard Server Not Detected"
            message="This folder/ZIP does not appear to be a standard Minecraft server (missing server.properties). Do you want to continue anyway?"
            onConfirm={onCreate}
            variant="warning"
            confirmText="Import Anyway"
          >
            {createButton}
          </ConfirmDropdown>
        ) : (
          createButton
        )}
      </div>
    </div>
  )
}
