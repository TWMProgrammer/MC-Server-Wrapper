import { useState, useEffect, useRef, useMemo } from 'react'
import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import Editor, { loader } from '@monaco-editor/react'
import { registerSkriptLanguage, getLanguageFromExtension } from '../utils/monaco'
import {
  X,
  Save,
  RefreshCw,
  FileText,
  Settings,
  AlertCircle,
  Terminal,
  Maximize2,
  Minimize2,
  ChevronRight,
  ChevronDown,
  Folder
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { InstalledPlugin } from '../types'
import { useToast } from '../hooks/useToast'
import { cn } from '../utils'
import { useAppSettings } from '../hooks/useAppSettings'

interface PluginConfigModalProps {
  plugin: InstalledPlugin;
  instanceId: string;
  onClose: () => void;
}

interface FileTreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileTreeNode[];
}

export function PluginConfigModal({ plugin, instanceId, onClose }: PluginConfigModalProps) {
  const [configs, setConfigs] = useState<string[]>([])
  const [configDir, setConfigDir] = useState<string>('')
  const [selectedConfig, setSelectedConfig] = useState<string | null>(null)
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const [content, setContent] = useState('')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [isMaximized, setIsMaximized] = useState(false)
  const [autoReload, setAutoReload] = useState(false)
  const { showToast } = useToast()
  const { settings } = useAppSettings()

  const editorRef = useRef<any>(null)

  useEffect(() => {
    loadConfigs()
  }, [plugin, instanceId])

  useEffect(() => {
    if (selectedConfig && configDir) {
      setContent('') // Clear content immediately when switching files to avoid ghosting
      loadFileContent(selectedConfig)
    }
  }, [selectedConfig, configDir])

  const handleEditorWillMount = (monaco: any) => {
    registerSkriptLanguage(monaco)
  }

  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor
  }

  const loadConfigs = async () => {
    setLoading(true)
    try {
      const result = await invoke<{ config_dir: string, files: string[] }>('list_plugin_configs', {
        instanceId,
        pluginName: plugin.name,
        pluginFilename: plugin.filename
      })
      setConfigs(result.files)
      setConfigDir(result.config_dir)
      if (result.files.length > 0) {
        // Find first file in the tree to select
        const firstFile = result.files.find(f => !f.endsWith('/')) || result.files[0]
        setSelectedConfig(firstFile)
      }
    } catch (err) {
      console.error('Failed to load plugin configs:', err)
      showToast('Failed to load configuration files', 'error')
    } finally {
      setLoading(false)
    }
  }

  const fileTree = useMemo(() => {
    const root: FileTreeNode[] = []

    configs.forEach(path => {
      const parts = path.split('/')
      let currentLevel = root

      parts.forEach((part, index) => {
        const isLast = index === parts.length - 1
        const currentPath = parts.slice(0, index + 1).join('/')

        let node = currentLevel.find(n => n.name === part)

        if (!node) {
          node = {
            name: part,
            path: currentPath,
            type: isLast ? 'file' : 'directory',
            children: isLast ? undefined : []
          }
          currentLevel.push(node)
        }

        if (node.children) {
          currentLevel = node.children
        }
      })
    })

    // Sort: directories first, then files
    const sortNodes = (nodes: FileTreeNode[]) => {
      nodes.sort((a, b) => {
        if (a.type !== b.type) {
          return a.type === 'directory' ? -1 : 1
        }
        return a.name.localeCompare(b.name)
      })
      nodes.forEach(node => {
        if (node.children) sortNodes(node.children)
      })
    }

    sortNodes(root)
    return root
  }, [configs])

  const loadFileContent = async (filename: string) => {
    if (!configDir) return
    setLoading(true)
    try {
      const relPath = `plugins/${configDir}/${filename}`
      const result = await invoke<string>('read_text_file', {
        instanceId,
        relPath
      })
      setContent(result)
    } catch (err) {
      console.error('Failed to load file content:', err)
      showToast('Failed to load file content', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleSave = async () => {
    if (!selectedConfig || !configDir) return
    setSaving(true)
    try {
      const relPath = `plugins/${configDir}/${selectedConfig}`
      const valueToSave = editorRef.current ? editorRef.current.getValue() : content;
      await invoke('save_text_file', {
        instanceId,
        relPath,
        content: valueToSave
      })
      showToast('Configuration saved successfully', 'success')

      if (autoReload) {
        handleReload()
      }
    } catch (err) {
      console.error('Failed to save config:', err)
      showToast('Failed to save configuration', 'error')
    } finally {
      setSaving(false)
    }
  }

  const handleReload = async () => {
    try {
      await invoke('send_command', {
        instanceId,
        command: `reload confirm`
      })
      showToast('Sent reload command to server', 'info')
    } catch (err) {
      console.error('Failed to send reload command:', err)
      showToast('Failed to send reload command (is the server running?)', 'error')
    }
  }

  const toggleFolder = (path: string) => {
    setExpandedFolders(prev => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }

  const renderFileTree = (nodes: FileTreeNode[], level: number = 0) => {
    return nodes.map(node => {
      const isExpanded = expandedFolders.has(node.path)
      const isSelected = selectedConfig === node.path

      if (node.type === 'directory') {
        return (
          <div key={node.path}>
            <button
              onClick={() => toggleFolder(node.path)}
              className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-gray-400 hover:bg-white/5 hover:text-gray-200 flex items-center gap-1.5 transition-all"
              style={{ paddingLeft: `${level * 12 + 8}px` }}
            >
              {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
              <Folder size={14} className="text-primary/60" />
              <span className="truncate">{node.name}</span>
            </button>
            {isExpanded && node.children && (
              <div className="mt-0.5">
                {renderFileTree(node.children, level + 1)}
              </div>
            )}
          </div>
        )
      }

      return (
        <button
          key={node.path}
          onClick={() => setSelectedConfig(node.path)}
          className={cn(
            "w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium transition-all flex items-center gap-1.5 group relative",
            isSelected
              ? "bg-primary text-white shadow-lg shadow-primary/20"
              : "text-gray-500 hover:bg-white/5 hover:text-gray-300",
            level > 0 && "ml-0"
          )}
          style={{ paddingLeft: `${level * 12 + 22}px` }}
        >
          <FileText size={14} className={cn(isSelected ? "text-white" : "text-gray-600 group-hover:text-gray-400")} />
          <span className="truncate">{node.name}</span>
          {isSelected && (
            <motion.div layoutId="active-file" className="absolute left-1 w-1 h-4 bg-white rounded-full" />
          )}
        </button>
      )
    })
  }

  const lineCount = content.split('\n').length
  const language = useMemo(() => getLanguageFromExtension(selectedConfig), [selectedConfig])

  return createPortal(
    <div 
      className="fixed inset-0 z-50 overflow-hidden"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
      <div className="w-full h-full flex items-center justify-center p-4 md:p-8">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className={cn(
            "bg-surface border border-white/10 shadow-2xl flex flex-col overflow-hidden relative transition-all duration-300",
            isMaximized ? "w-full h-full" : "w-[80%] h-[80%] rounded-3xl"
          )}
        >
        {/* Header */}
        <div className="flex items-center justify-between px-8 py-6 border-b border-white/5 bg-[#1a1a1a]">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-primary/10 rounded-2xl text-primary">
              <Settings size={24} />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white leading-tight">
                Config: {plugin.name}
              </h2>
              <p className="text-[10px] text-gray-500 uppercase tracking-widest font-bold mt-0.5">
                Plugin Configuration Editor
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => setIsMaximized(!isMaximized)}
              className="p-2.5 hover:bg-white/5 rounded-xl transition-all text-gray-400 hover:text-white"
              title={isMaximized ? "Minimize" : "Maximize"}
            >
              {isMaximized ? <Minimize2 size={20} /> : <Maximize2 size={20} />}
            </button>
            <button
              onClick={onClose}
              className="p-2.5 hover:bg-red-500/10 rounded-xl transition-all text-gray-400 hover:text-red-500"
            >
              <X size={24} />
            </button>
          </div>
        </div>

        <div className="flex-1 flex min-h-0">
          {/* Sidebar */}
          <div className="w-72 border-r border-white/5 bg-[#141414] flex flex-col shrink-0">
            <div className="p-6 overflow-y-auto custom-scrollbar flex-1">
              <div className="flex items-center gap-2 mb-4 px-2">
                <FileText size={16} className="text-primary" />
                <h3 className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-500">
                  Files
                </h3>
              </div>

              <div className="space-y-0.5">
                {configs.length === 0 && !loading ? (
                  <div className="px-4 py-8 text-center bg-white/5 rounded-2xl border border-white/5">
                    <AlertCircle size={24} className="mx-auto text-gray-600 mb-2" />
                    <p className="text-xs text-gray-500 leading-relaxed">
                      No config files found.<br />
                      <span className="opacity-50 text-[10px]">Is the plugin folder named correctly?</span>
                    </p>
                  </div>
                ) : (
                  renderFileTree(fileTree)
                )}
              </div>
            </div>

            <div className="p-6 border-t border-white/5 bg-[#1a1a1a]">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Terminal size={14} className="text-primary" />
                  <span className="text-[10px] font-bold text-gray-500 uppercase tracking-wider">Reload</span>
                </div>
                <button
                  onClick={() => setAutoReload(!autoReload)}
                  className={cn(
                    "w-10 h-5 rounded-full transition-all relative border",
                    autoReload ? "bg-primary border-primary" : "bg-black/40 border-white/10"
                  )}
                >
                  <motion.div
                    animate={{ x: autoReload ? 20 : 2 }}
                    className="absolute top-1 w-2.5 h-2.5 rounded-full bg-white shadow-sm"
                  />
                </button>
              </div>
              <p className="text-[10px] text-gray-500 leading-relaxed">
                Automatically send <b>/reload confirm</b> after saving changes.
              </p>
            </div>
          </div>

          {/* Editor Area */}
          <div className="flex-1 flex flex-col bg-[#1e1e1e] relative">
            {loading ? (
              <div className="absolute inset-0 z-10 bg-black/20 backdrop-blur-[2px] flex items-center justify-center">
                <RefreshCw size={32} className="animate-spin text-primary opacity-50" />
              </div>
            ) : null}

            <div className="flex-1 overflow-hidden relative">
              <Editor
                height="100%"
                language={language}
                theme="vs-dark"
                value={content}
                beforeMount={handleEditorWillMount}
                onMount={handleEditorDidMount}
                onChange={(value) => setContent(value || '')}
                options={{
                  fontSize: 14,
                  fontFamily: 'JetBrains Mono, Fira Code, monospace',
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  wordWrap: 'off',
                  lineNumbers: 'on',
                  renderWhitespace: 'selection',
                  scrollbar: {
                    vertical: 'visible',
                    horizontal: 'visible',
                    useShadows: false,
                    verticalScrollbarSize: 10,
                    horizontalScrollbarSize: 10,
                  },
                  padding: { top: 20, bottom: 20 },
                  automaticLayout: true,
                  backgroundColor: '#1e1e1e',
                  rules: [
                    { token: 'comment', foreground: '6A9955' },
                    { token: 'keyword', foreground: '569CD6' },
                    { token: 'string', foreground: 'CE9178' },
                    { token: 'variable', foreground: '9CDCFE' },
                  ]
                } as any}
              />
            </div>

            {/* Footer Actions */}
            <div className="p-6 border-t border-white/5 bg-[#1a1a1a] flex items-center justify-between">
              <div className="flex items-center gap-4 text-xs text-gray-500">
                <span className="flex items-center gap-1.5 max-w-[300px] truncate">
                  <FileText size={14} className="opacity-50 shrink-0" />
                  {selectedConfig || 'No file selected'}
                </span>
                <span className="w-px h-3 bg-white/10 shrink-0" />
                <span className="shrink-0">{lineCount} lines</span>
              </div>

              <div className="flex items-center gap-3">
                <button
                  onClick={handleReload}
                  className="flex items-center gap-2 px-5 py-2.5 bg-white/5 hover:bg-white/10 text-white rounded-xl text-sm font-bold transition-all"
                >
                  <RefreshCw size={18} />
                  Manual Reload
                </button>
                <button
                  onClick={handleSave}
                  disabled={saving || !selectedConfig}
                  className="flex items-center gap-2 px-8 py-2.5 bg-primary text-white rounded-xl text-sm font-bold shadow-xl shadow-primary/20 hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {saving ? (
                    <RefreshCw size={18} className="animate-spin" />
                  ) : (
                    <Save size={18} />
                  )}
                  Save Changes
                </button>
              </div>
            </div>
          </div>
        </div>
        </motion.div>
      </div>
    </div>,
    document.body
  )
}


