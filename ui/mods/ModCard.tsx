import { motion } from 'framer-motion'
import { Check, Package, User, Download, ExternalLink, Star } from 'lucide-react'
import { Project } from '../types'

interface ModCardProps {
  project: Project;
  isSelected: boolean;
  onSelect: (project: Project) => void;
  onShowDetails: (project: Project) => void;
}

export function ModCard({
  project,
  isSelected,
  onSelect,
  onShowDetails
}: ModCardProps) {
  return (
    <motion.div
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
          <img src={project.icon_url} alt="" className="w-16 h-16 rounded-2xl object-cover bg-black/20 shadow-xl" />
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
            <span className="text-[10px] font-bold truncate">{(project.downloads / 1000).toFixed(1)}k</span>
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
