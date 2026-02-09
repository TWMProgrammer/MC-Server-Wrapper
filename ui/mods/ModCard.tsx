import { motion } from 'framer-motion'
import { Check, Package, User, Download, ExternalLink, Star } from 'lucide-react'
import { Project } from '../types'
import { useAssetCache } from '../hooks/useAssetCache'
import { useInView } from '../hooks/useInView'
import { formatNumber } from '../utils'

interface ModCardProps {
  project: Project;
  isSelected: boolean;
  onSelect: (project: Project) => void;
  onShowDetails: (project: Project) => void;
  viewMode?: 'grid' | 'list';
}

export function ModCard({
  project,
  isSelected,
  onSelect,
  onShowDetails,
  viewMode = 'grid'
}: ModCardProps) {
  const [ref, isInView] = useInView({ rootMargin: '200px' });
  const { localUrl: iconUrl } = useAssetCache(project.icon_url, isInView);

  if (viewMode === 'list') {
    return (
      <motion.div
        ref={ref as any}
        layout
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className={`relative bg-surface border transition-all group flex items-center p-4 rounded-2xl gap-4 ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
          }`}
      >
        <div className="relative shrink-0">
          {project.icon_url ? (
            <img src={iconUrl || project.icon_url} alt="" className="w-12 h-12 rounded-xl object-cover bg-black/20 shadow-lg" />
          ) : (
            <div className="w-12 h-12 rounded-xl bg-primary/10 text-primary flex items-center justify-center shadow-lg">
              <Package size={24} />
            </div>
          )}
          {isSelected && (
            <div className="absolute -top-2 -right-2 bg-primary text-white p-1 rounded-full shadow-lg z-10 border-2 border-[#0a0a0c]">
              <Check size={8} strokeWidth={4} />
            </div>
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3">
            <h3 className="font-bold text-white truncate text-base group-hover:text-primary transition-colors">
              {project.title}
            </h3>
            <div className="flex items-center gap-1.5 px-2 py-0.5 bg-white/5 rounded-lg shrink-0">
              <div className={`w-1.5 h-1.5 rounded-full ${project.provider === 'Modrinth' ? 'bg-green-500' : 'bg-orange-500'}`} />
              <span className="text-[9px] font-black uppercase tracking-widest text-gray-500">{project.provider}</span>
            </div>
          </div>
          <p className="text-sm text-gray-400 line-clamp-1 font-medium mt-0.5">
            {project.description}
          </p>
        </div>

        <div className="flex items-center gap-6 shrink-0 ml-4 px-6 border-l border-white/5">
          <div className="flex flex-col items-center gap-1">
            <Download size={14} className="text-primary" />
            <span className="text-[10px] font-bold text-gray-500">{formatNumber(project.downloads)}</span>
          </div>
          <div className="flex flex-col items-center gap-1">
            <Star size={14} className="text-primary" />
            <span className="text-[10px] font-bold text-gray-500">{(project.downloads / 5000).toFixed(0)}</span>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0 ml-2">
          <button
            onClick={() => onShowDetails(project)}
            className="p-2.5 bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white rounded-xl transition-all"
          >
            <ExternalLink size={18} />
          </button>
          <button
            onClick={() => onSelect(project)}
            className={`px-5 py-2.5 rounded-xl text-xs font-black transition-all ${isSelected
              ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
              : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-105'
              }`}
          >
            {isSelected ? 'Remove' : 'Select'}
          </button>
        </div>
      </motion.div>
    )
  }

  return (
    <motion.div
      ref={ref as any}
      layout
      data-testid="mod-card"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={`relative bg-surface border transition-all group flex flex-col p-6 rounded-[2rem] h-56 ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
        }`}
    >
      {isSelected && (
        <div className="absolute top-3 right-3 bg-primary text-white p-1.5 rounded-full shadow-lg z-10 border-2 border-[#0a0a0c]">
          <Check size={12} strokeWidth={4} />
        </div>
      )}

      <div className="flex items-start gap-4 mb-4">
        {project.icon_url ? (
          <img src={iconUrl || project.icon_url} alt="" className="w-16 h-16 rounded-2xl object-cover bg-black/20 shadow-xl" />
        ) : (
          <div className="w-16 h-16 rounded-2xl bg-primary/10 text-primary flex items-center justify-center shadow-xl">
            <Package size={32} />
          </div>
        )}
        <div className="flex-1 min-w-0">
          <h3 className="font-bold text-white truncate text-base group-hover:text-primary transition-colors">
            {project.title}
          </h3>
          <div className="flex items-center gap-2 mt-1">
            <div className={`w-2 h-2 rounded-full ${project.provider === 'Modrinth' ? 'bg-green-500' : 'bg-orange-500'}`} />
            <span className="text-[10px] font-black uppercase tracking-widest text-gray-500">{project.provider}</span>
          </div>
        </div>
      </div>

      <p className="text-sm text-gray-400 line-clamp-2 mb-4 font-medium leading-relaxed flex-1">
        {project.description}
      </p>

      <div className="flex items-center justify-between pt-4 border-t border-white/5 gap-2">
        <div className="flex items-center gap-3 min-w-0 overflow-hidden">
          <div className="flex items-center gap-1 text-gray-500 shrink-0">
            <Download size={14} className="text-primary" />
            <span className="text-[10px] font-bold truncate">{formatNumber(project.downloads)}</span>
          </div>
          <div className="flex items-center gap-1 text-gray-500 shrink-0">
            <Star size={14} className="text-primary" />
            <span className="text-[10px] font-bold truncate">{(project.downloads / 5000).toFixed(0)}</span>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <button
            onClick={() => onShowDetails(project)}
            className="p-2 bg-white/5 hover:bg-white/10 text-gray-400 hover:text-white rounded-xl transition-all"
          >
            <ExternalLink size={16} />
          </button>
          <button
            onClick={() => onSelect(project)}
            className={`px-4 py-2 rounded-xl text-xs font-black transition-all ${isSelected
              ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
              : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-105'
              }`}
          >
            {isSelected ? 'Remove' : 'Select'}
          </button>
        </div>
      </div>
    </motion.div>
  )
}
