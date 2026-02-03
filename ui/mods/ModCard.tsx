import { motion } from 'framer-motion'
import { Check, Package, User, Download, ChevronRight } from 'lucide-react'
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
      className={`relative bg-surface border transition-all group flex flex-col p-6 rounded-[2rem] ${isSelected ? 'border-primary bg-primary/5' : 'border-white/5 hover:border-white/20'
        }`}
    >
      {isSelected && (
        <div className="absolute -top-2 -right-2 bg-primary text-white p-1.5 rounded-full shadow-lg z-10">
          <Check size={16} strokeWidth={3} />
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
          <h3 className="font-black text-white truncate text-lg">
            {project.title}
          </h3>
          <div className="flex items-center gap-2 text-xs text-gray-500 mt-1 font-medium min-w-0">
            <span className="flex items-center gap-1 min-w-0 flex-1">
              <User size={12} className="text-primary shrink-0" />
              <span className="truncate">{project.author || 'Unknown'}</span>
            </span>
            <span className="flex items-center gap-1 shrink-0">
              <Download size={12} className="text-primary" />
              {project.downloads.toLocaleString()}
            </span>
          </div>
        </div>
      </div>

      <p className="text-sm text-gray-400 truncate mb-6 font-medium leading-relaxed">
        {project.description}
      </p>

      <div className="flex items-center gap-2 mt-auto">
        <button
          onClick={() => onShowDetails(project)}
          className="flex-1 py-3 bg-white/5 hover:bg-white/10 text-gray-300 rounded-xl text-sm font-bold transition-all flex items-center justify-center gap-2"
        >
          Details
          <ChevronRight size={16} />
        </button>
        <button
          onClick={() => onSelect(project)}
          className={`px-4 py-3 rounded-xl text-sm font-black transition-all flex items-center gap-2 ${isSelected
            ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
            : 'bg-primary text-white shadow-lg shadow-primary/20 hover:scale-[1.02]'
            }`}
        >
          {isSelected ? 'Remove' : 'Select'}
        </button>
      </div>
    </motion.div>
  )
}
