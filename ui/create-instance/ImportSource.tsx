import { motion, AnimatePresence } from 'framer-motion'
import { HardDrive, FileWarning, Folder, FileArchive, ChevronDown, Cpu, Zap, FileCode, Terminal } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { cn } from '../utils'
import { useState } from 'react'
import { Select } from '../components/Select'
import { ArchiveFileTree } from './ArchiveFileTree'
import { useEffect } from 'react'

interface ParsedScriptInfo {
  min_ram?: number;
  min_ram_unit?: string;
  max_ram?: number;
  max_ram_unit?: string;
  jvm_args: string[];
  jar_name?: string;
  server_args: string[];
  has_restart_loop: boolean;
  java_path?: string;
}

interface ImportSourceProps {
  importSourcePath: string | null;
  setImportSourcePath: (path: string | null) => void;
  importServerType: string;
  setImportServerType: (type: string) => void;
  availableJars: string[];
  setAvailableJars: (jars: string[]) => void;
  selectedJar: string | null;
  setSelectedJar: (jar: string | null) => void;
  availableScripts: string[];
  setAvailableScripts: (scripts: string[]) => void;
  selectedScript: string | null;
  setSelectedScript: (script: string | null) => void;
  serverPropertiesExists: boolean;
  setServerPropertiesExists: (exists: boolean) => void;
  bypassServerPropertiesCheck: boolean;
  setBypassServerPropertiesCheck: (bypass: boolean) => void;
  rootWithinZip: string | null;
  setRootWithinZip: (path: string | null) => void;
}

export function ImportSource({
  importSourcePath,
  setImportSourcePath,
  importServerType,
  setImportServerType,
  availableJars,
  setAvailableJars,
  selectedJar,
  setSelectedJar,
  availableScripts,
  setAvailableScripts,
  selectedScript,
  setSelectedScript,
  serverPropertiesExists,
  setServerPropertiesExists,
  bypassServerPropertiesCheck,
  setBypassServerPropertiesCheck,
  rootWithinZip,
  setRootWithinZip
}: ImportSourceProps) {
  const [loading, setLoading] = useState(false);
  const [scriptInfo, setScriptInfo] = useState<ParsedScriptInfo | null>(null);
  const [parsingScript, setParsingScript] = useState(false);

  useEffect(() => {
    if (importSourcePath) {
      processSelection(importSourcePath, rootWithinZip);
    }
  }, [importSourcePath, rootWithinZip]);

  useEffect(() => {
    if (selectedScript && importSourcePath) {
      handlePreviewScript(selectedScript);
    } else {
      setScriptInfo(null);
    }
  }, [selectedScript, importSourcePath, rootWithinZip]);

  const handlePreviewScript = async (script: string) => {
    setParsingScript(true);
    try {
      const info = await invoke<ParsedScriptInfo>('preview_script_import', {
        sourcePath: importSourcePath,
        scriptPath: script,
        rootWithinZip
      });
      setScriptInfo(info);

      // Auto-select JAR if found in script and not already selected
      if (info.jar_name && !selectedJar && availableJars.includes(info.jar_name)) {
        setSelectedJar(info.jar_name);
      }
    } catch (e) {
      console.error('Failed to preview script:', e);
      setScriptInfo(null);
    } finally {
      setParsingScript(false);
    }
  };

  const processSelection = async (path: string, root: string | null) => {
    setLoading(true);
    try {
      // Auto-detect server type
      const detectedType = await invoke<string>('detect_server_type', { sourcePath: path, rootWithinZip: root });
      if (detectedType !== 'unknown') {
        setImportServerType(detectedType);
      } else {
        setImportServerType('vanilla');
      }

      const [jars, scripts] = await Promise.all([
        invoke<string[]>('list_jars_in_source', { sourcePath: path, rootWithinZip: root }),
        invoke<string[]>('list_scripts_in_source', { sourcePath: path, rootWithinZip: root })
      ]);

      setAvailableJars(jars);
      if (jars.length === 1) {
        setSelectedJar(jars[0]);
      } else if (!jars.includes(selectedJar || '')) {
        setSelectedJar(null);
      }

      setAvailableScripts(scripts);
      if (scripts.length === 1) {
        setSelectedScript(scripts[0]);
      } else if (!scripts.includes(selectedScript || '')) {
        setSelectedScript(null);
      }

      const exists = await invoke<boolean>('check_server_properties_exists', { sourcePath: path, rootWithinZip: root });
      setServerPropertiesExists(exists);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handlePickFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === 'string') {
      setRootWithinZip(null);
      setImportSourcePath(selected);
    }
  };

  const handlePickArchive = async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [{ name: 'Archives', extensions: ['zip', '7z'] }]
    });
    if (selected && typeof selected === 'string') {
      setRootWithinZip(null);
      setImportSourcePath(selected);
    }
  };

  return (
    <div className="flex-1 overflow-auto p-8 custom-scrollbar">
      <div className="max-w-2xl mx-auto space-y-8">
        <div className="space-y-4">
          <h2 className="text-sm font-black text-gray-500 dark:text-white/40 uppercase tracking-[0.2em]">Select Source</h2>
          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={handlePickFolder}
              className={cn(
                "flex flex-col items-center gap-4 p-6 rounded-2xl border transition-all",
                importSourcePath && !importSourcePath.endsWith('.zip')
                  ? "bg-primary/10 border-primary text-primary"
                  : "bg-black/5 dark:bg-white/[0.02] border-black/5 dark:border-white/5 hover:bg-black/10 dark:hover:bg-white/[0.05]"
              )}
            >
              <Folder size={32} />
              <div className="text-center">
                <div className="font-bold text-sm">Select Folder</div>
                <div className="text-[10px] opacity-50 uppercase font-black tracking-widest mt-1">Existing Server</div>
              </div>
            </button>
            <button
              onClick={handlePickArchive}
              className={cn(
                "flex flex-col items-center gap-4 p-6 rounded-2xl border transition-all",
                (importSourcePath?.endsWith('.zip') || importSourcePath?.endsWith('.7z'))
                  ? "bg-primary/10 border-primary text-primary"
                  : "bg-black/5 dark:bg-white/[0.02] border-black/5 dark:border-white/5 hover:bg-black/10 dark:hover:bg-white/[0.05]"
              )}
            >
              <FileArchive size={32} />
              <div className="text-center">
                <div className="font-bold text-sm">Select Archive</div>
                <div className="text-[10px] opacity-50 uppercase font-black tracking-widest mt-1">Server Backup/Archive (ZIP, 7z)</div>
              </div>
            </button>
          </div>

          {importSourcePath && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="p-4 rounded-xl bg-black/5 dark:bg-white/[0.02] border border-black/5 dark:border-white/5 space-y-3"
            >
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-black/10 dark:bg-black/40">
                  {(importSourcePath.endsWith('.zip') || importSourcePath.endsWith('.7z')) ? <FileArchive size={16} /> : <Folder size={16} />}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-[10px] font-black uppercase tracking-widest text-gray-500">Selected Source</div>
                  <div className="text-xs font-medium truncate">{importSourcePath}</div>
                </div>
              </div>

              {!serverPropertiesExists && (
                <div className="flex flex-col gap-3">
                  {!bypassServerPropertiesCheck && (
                    <div className="flex items-start justify-between gap-3 p-3 rounded-lg bg-accent-amber/10 border border-accent-amber/20 text-accent-amber">
                      <div className="flex items-start gap-3">
                        <FileWarning size={16} className="mt-0.5 shrink-0" />
                        <div className="text-[11px] font-medium leading-relaxed">
                          This source doesn't seem to be a standard Minecraft server (missing server.properties).
                        </div>
                      </div>
                      <button
                        onClick={() => setBypassServerPropertiesCheck(true)}
                        className="text-[9px] font-black uppercase tracking-widest bg-accent-amber/20 hover:bg-accent-amber/30 px-2 py-1 rounded transition-colors whitespace-nowrap"
                      >
                        I know what I'm doing
                      </button>
                    </div>
                  )}

                  {(importSourcePath.endsWith('.zip') || importSourcePath.endsWith('.7z')) && (
                    <ArchiveFileTree
                      archivePath={importSourcePath}
                      onSelectRoot={setRootWithinZip}
                      selectedRoot={rootWithinZip}
                    />
                  )}
                </div>
              )}
            </motion.div>
          )}
        </div>

        {importSourcePath && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="space-y-6"
          >
            <div className="space-y-4">
              <h2 className="text-sm font-black text-gray-500 dark:text-white/40 uppercase tracking-[0.2em]">Server Configuration</h2>
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-xs font-bold text-gray-400 uppercase tracking-wider ml-1">Server Type</label>
                  <Select
                    value={importServerType}
                    onChange={setImportServerType}
                    options={[
                      { value: 'vanilla', label: 'Vanilla' },
                      { value: 'paper', label: 'Paper/Spigot/Bukkit' },
                      { value: 'forge', label: 'Forge' },
                      { value: 'fabric', label: 'Fabric' },
                      { value: 'quilt', label: 'Quilt' },
                      { value: 'custom', label: 'Custom/Other' },
                    ]}
                    placeholder="Select server type..."
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold text-gray-400 uppercase tracking-wider ml-1">Executable JAR (Mandatory)</label>
                  <Select
                    value={selectedJar || ''}
                    onChange={setSelectedJar}
                    options={availableJars.map(jar => ({ value: jar, label: jar }))}
                    placeholder="Select a JAR file..."
                    loading={loading}
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-gray-400 uppercase tracking-wider ml-1">Import settings from Script (Optional)</label>
                <Select
                  value={selectedScript || ''}
                  onChange={setSelectedScript}
                  options={[
                    { value: '', label: 'No script (use default settings)' },
                    ...availableScripts.map(script => ({ value: script, label: script }))
                  ]}
                  placeholder="Select a .bat or .cmd file..."
                  loading={loading}
                />
                <p className="text-[10px] text-gray-500 ml-1">
                  Selecting a startup script will attempt to parse its RAM, JVM arguments, and JAR settings.
                </p>
              </div>

              <AnimatePresence>
                {scriptInfo && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    exit={{ opacity: 0, height: 0 }}
                    className="overflow-hidden"
                  >
                    <div className="p-4 rounded-xl bg-primary/5 border border-primary/10 space-y-4">
                      <div className="flex items-center gap-2 text-primary">
                        <Terminal size={14} />
                        <span className="text-xs font-black uppercase tracking-widest">Extracted from {selectedScript}</span>
                      </div>

                      <div className="grid grid-cols-2 gap-4">
                        {(scriptInfo.min_ram || scriptInfo.max_ram) && (
                          <div className="space-y-1">
                            <div className="flex items-center gap-1.5 text-gray-500 dark:text-white/40">
                              <Cpu size={12} />
                              <span className="text-[11px] font-bold uppercase tracking-wider">Memory</span>
                            </div>
                            <div className="text-sm font-medium">
                              {scriptInfo.min_ram}{scriptInfo.min_ram_unit} - {scriptInfo.max_ram}{scriptInfo.max_ram_unit}
                            </div>
                          </div>
                        )}

                        {scriptInfo.jar_name && (
                          <div className="space-y-1">
                            <div className="flex items-center gap-1.5 text-gray-500 dark:text-white/40">
                              <FileCode size={12} />
                              <span className="text-[11px] font-bold uppercase tracking-wider">Target JAR</span>
                            </div>
                            <div className="text-sm font-medium truncate">
                              {scriptInfo.jar_name}
                            </div>
                          </div>
                        )}

                        {scriptInfo.jvm_args.length > 0 && (
                          <div className="space-y-1 col-span-2">
                            <div className="flex items-center gap-1.5 text-gray-500 dark:text-white/40">
                              <Terminal size={12} />
                              <span className="text-[11px] font-bold uppercase tracking-wider">JVM Flags</span>
                            </div>
                            <div className="flex flex-wrap gap-1">
                              {scriptInfo.jvm_args.map((arg, i) => (
                                <span key={i} className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/5 text-[10px] font-mono">
                                  {arg}
                                </span>
                              ))}
                            </div>
                          </div>
                        )}

                        {scriptInfo.java_path && (
                          <div className="space-y-1 col-span-2">
                            <div className="flex items-center gap-1.5 text-gray-500 dark:text-white/40">
                              <Zap size={12} />
                              <span className="text-[11px] font-bold uppercase tracking-wider">Custom Java Path</span>
                            </div>
                            <div className="text-sm font-mono p-2 rounded bg-black/10 dark:bg-white/5 truncate">
                              {scriptInfo.java_path}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>

              {availableJars.length === 0 && !loading && (
                <div className="text-[10px] text-accent-rose font-bold uppercase tracking-wider ml-1 mt-1">
                  No .jar files found in source!
                </div>
              )}
            </div>
          </motion.div>
        )}
      </div>
    </div>
  );
}
