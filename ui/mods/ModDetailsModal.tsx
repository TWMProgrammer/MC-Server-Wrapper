import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  X,
  Download,
  ExternalLink,
  User,
  Calendar,
  Tag,
  Package,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  Star,
  Cpu
} from 'lucide-react'
import { Project } from '../types'
import { useAppSettings } from '../hooks/useAppSettings'

interface ModDetailsModalProps {
  project: Project;
  instanceId: string;
  onClose: () => void;
  onInstall: () => void;
  isSelected: boolean;
}

export function ModDetailsModal({
  project,
  onClose,
  onInstall,
  isSelected
}: ModDetailsModalProps) {
  const { settings } = useAppSettings()

  return createPortal(
    <div
      className="fixed inset-0 z-[100] overflow-hidden"
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
          className="relative w-[75%] h-[90%] bg-[#0a0a0c] border border-white/10 rounded-3xl shadow-2xl overflow-hidden flex flex-col"
        >
          {/* Header */}
          <div className="p-6 border-b border-white/5 flex items-start justify-between bg-white/[0.02]">
            <div className="flex items-center gap-5">
              {project.icon_url ? (
                <img src={project.icon_url} alt="" className="w-20 h-20 rounded-2xl object-cover bg-black/40 shadow-xl" />
              ) : (
                <div className="w-20 h-20 rounded-2xl bg-primary/10 text-primary flex items-center justify-center shadow-xl">
                  <Package size={40} />
                </div>
              )}
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-3 mb-1">
                  <h2 className="text-2xl font-bold text-white truncate">{project.title}</h2>
                  <span className={`text-xs px-2 py-0.5 rounded-full font-bold uppercase tracking-wider ${project.provider === 'Modrinth' ? 'bg-green-500/10 text-green-500' : 'bg-orange-500/10 text-orange-500'
                    }`}>
                    {project.provider}
                  </span>
                </div>
                <div className="flex flex-wrap items-center gap-4 text-sm text-gray-400">
                  <span className="flex items-center gap-1.5">
                    <User size={14} className="text-primary" />
                    {project.author}
                  </span>
                  <span className="flex items-center gap-1.5">
                    <Download size={14} className="text-primary" />
                    {project.downloads.toLocaleString()} downloads
                  </span>
                  {project.categories && project.categories.length > 0 && (
                    <span className="flex items-center gap-1.5">
                      <Tag size={14} className="text-primary" />
                      {project.categories.slice(0, 3).join(', ')}
                    </span>
                  )}
                </div>
              </div>
            </div>

            <button
              onClick={onClose}
              className="p-2 hover:bg-white/10 text-gray-500 hover:text-white rounded-full transition-colors"
            >
              <X size={24} />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
              <div className="lg:col-span-2 space-y-8">
                <section>
                  <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
                    <Tag size={18} className="text-primary" />
                    Description
                  </h3>
                  <div className="text-gray-300 leading-relaxed bg-white/5 p-6 rounded-2xl border border-white/5">
                    {project.description}
                    <p className="mt-4 text-sm text-gray-500 italic">
                      Note: Detailed Markdown descriptions are currently only available on the provider's website.
                    </p>
                  </div>
                </section>

                <section className="bg-primary/5 border border-primary/10 rounded-2xl p-6">
                  <div className="flex items-start gap-4">
                    <div className="p-3 bg-primary/20 rounded-xl text-primary">
                      <AlertCircle size={24} />
                    </div>
                    <div>
                      <h4 className="font-bold text-white mb-1">Installation Note</h4>
                      <p className="text-sm text-gray-400 leading-relaxed">
                        This will download the latest compatible version of <b>{project.title}</b> directly into your server's mods folder. Ensure your mod loader and game version match the project's requirements.
                      </p>
                    </div>
                  </div>
                </section>
              </div>

              <div className="space-y-6">
                <div className="bg-white/5 border border-white/5 rounded-2xl p-6 space-y-4">
                  <h3 className="font-bold text-white flex items-center gap-2">
                    <Package size={18} className="text-primary" />
                    Project Info
                  </h3>

                  <div className="space-y-3">
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-500">Provider</span>
                      <span className="text-gray-300">{project.provider}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-500">ID</span>
                      <span className="text-gray-300 font-mono text-[10px]">{project.id}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-500">Slug</span>
                      <span className="text-gray-300">{project.slug}</span>
                    </div>
                  </div>

                  <div className="pt-4 border-t border-white/5">
                    <button
                      onClick={async () => {
                        const url = project.provider === 'Modrinth'
                          ? `https://modrinth.com/mod/${project.slug}`
                          : `https://www.curseforge.com/minecraft/mc-mods/${project.slug}`;
                        try {
                          await openUrl(url);
                        } catch (err) {
                          console.error('Failed to open URL:', err);
                        }
                      }}
                      className="w-full py-2.5 bg-white/5 hover:bg-white/10 text-white rounded-xl text-sm font-medium transition-all flex items-center justify-center gap-2"
                    >
                      <ExternalLink size={16} />
                      View on {project.provider}
                    </button>
                  </div>
                </div>

                <button
                  onClick={onInstall}
                  className={`w-full py-4 rounded-2xl font-bold shadow-lg transition-all flex items-center justify-center gap-3 ${isSelected
                    ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20 shadow-red-500/10'
                    : 'bg-primary text-white shadow-primary/20 hover:bg-primary/90'
                    }`}
                >
                  {isSelected ? (
                    <>
                      <X size={20} />
                      Deselect Mod
                    </>
                  ) : (
                    <>
                      <CheckCircle2 size={20} />
                      Select for Download
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </div>,
    document.body
  )
}
