import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Folder, File, ChevronRight, ChevronDown, CheckCircle2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ZipEntry } from './types';

interface ArchiveFileTreeProps {
  archivePath: string;
  onSelectRoot: (path: string | null) => void;
  selectedRoot: string | null;
}

interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: TreeNode[];
}

export function ArchiveFileTree({ archivePath, onSelectRoot, selectedRoot }: ArchiveFileTreeProps) {
  const [nodes, setNodes] = useState<TreeNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadArchiveContents();
  }, [archivePath]);

  const loadArchiveContents = async () => {
    try {
      setLoading(true);
      const entries = await invoke<ZipEntry[]>('list_archive_contents', { archivePath });
      const tree = buildTree(entries);
      setNodes(tree);
      
      // Auto-expand first level
      const firstLevel = new Set<string>();
      tree.forEach(node => {
        if (node.is_dir) firstLevel.add(node.path);
      });
      setExpanded(firstLevel);
    } catch (e) {
      console.error('Failed to load archive contents', e);
    } finally {
      setLoading(false);
    }
  };

  const buildTree = (entries: ZipEntry[]): TreeNode[] => {
    const rootNodes: TreeNode[] = [];
    const map: Record<string, TreeNode> = {};

    // Sort entries by path depth to ensure parents are processed before children
    const sortedEntries = [...entries].sort((a, b) => {
        const aDepth = a.path.split('/').filter(Boolean).length;
        const bDepth = b.path.split('/').filter(Boolean).length;
        return aDepth - bDepth;
    });

    sortedEntries.forEach(entry => {
      const parts = entry.path.split('/').filter(p => p.length > 0);
      const node: TreeNode = {
        name: entry.name,
        path: entry.path,
        is_dir: entry.is_dir,
        children: []
      };

      if (parts.length === 1 || (parts.length === 2 && entry.is_dir && entry.path.endsWith('/'))) {
          // It's a top level node
          // Check if it's already in rootNodes (might happen with some formats)
          if (!rootNodes.find(n => n.path === node.path)) {
              rootNodes.push(node);
          }
      } else {
        // Find parent path
        // For a path like "a/b/c", parent is "a/b/"
        // For a path like "a/b/", parent is "a/"
        const pathParts = entry.path.split('/').filter(Boolean);
        pathParts.pop(); // remove last part
        
        const parentPath = pathParts.length > 0 ? pathParts.join('/') + '/' : '';
        
        if (parentPath && map[parentPath]) {
          map[parentPath].children.push(node);
        } else if (!parentPath) {
            // Should be handled by the top-level check, but just in case
            if (!rootNodes.find(n => n.path === node.path)) {
                rootNodes.push(node);
            }
        }
      }
      map[entry.path] = node;
    });

    return rootNodes;
  };

  const toggleExpand = (path: string) => {
    const next = new Set(expanded);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    setExpanded(next);
  };

  const renderNode = (node: TreeNode, depth: number = 0) => {
    const isExpanded = expanded.has(node.path);
    const isSelected = selectedRoot === node.path || (selectedRoot === null && node.path === '');
    
    return (
      <div key={node.path}>
        <div 
          className={`flex items-center py-1 px-2 rounded-md cursor-pointer transition-colors group ${
            isSelected ? 'bg-amber-500/20 text-amber-500' : 'hover:bg-white/5 text-white/70'
          }`}
          style={{ paddingLeft: `${depth * 1.25 + 0.5}rem` }}
          onClick={() => {
            if (node.is_dir) {
              onSelectRoot(node.path);
            }
          }}
        >
          <div className="flex items-center flex-1 min-w-0">
            {node.is_dir && (
              <button 
                onClick={(e) => {
                  e.stopPropagation();
                  toggleExpand(node.path);
                }}
                className="p-0.5 hover:bg-white/10 rounded mr-1 flex-shrink-0"
              >
                {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
              </button>
            )}
            {!node.is_dir && <div className="w-5 flex-shrink-0" />}
            {node.is_dir ? (
              <Folder size={16} className={`mr-2 flex-shrink-0 ${isSelected ? 'text-amber-500' : 'text-amber-500/60'}`} />
            ) : (
              <File size={16} className="mr-2 flex-shrink-0 text-white/30" />
            )}
            <span className="text-sm truncate">{node.name}</span>
          </div>
          {isSelected && <CheckCircle2 size={14} className="ml-2 flex-shrink-0 text-amber-500" />}
        </div>
        
        <AnimatePresence initial={false}>
          {node.is_dir && isExpanded && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: 'auto', opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: 0.2, ease: "easeInOut" }}
              className="overflow-hidden"
            >
              {node.children.map(child => renderNode(child, depth + 1))}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-amber-500"></div>
      </div>
    );
  }

  const extension = archivePath.split('.').pop()?.toUpperCase() || 'Archive';

  return (
    <div className="mt-4 border border-white/10 rounded-lg bg-black/40 overflow-hidden flex flex-col">
      <div className="bg-white/5 px-4 py-2.5 border-b border-white/10 flex items-center justify-between">
        <div className="flex flex-col">
          <span className="text-[10px] font-bold text-white/40 uppercase tracking-widest">{extension} Contents</span>
          <span className="text-xs text-white/70">Select the folder containing server.properties</span>
        </div>
        {selectedRoot && (
          <button 
            onClick={(e) => {
              e.stopPropagation();
              onSelectRoot(null);
            }}
            className="text-[10px] font-bold text-amber-500 hover:text-amber-400 transition-colors uppercase tracking-wider bg-amber-500/10 px-2 py-1 rounded"
          >
            Reset
          </button>
        )}
      </div>
      <div className="p-2 max-h-[300px] overflow-y-auto custom-scrollbar">
        {nodes.length === 0 ? (
          <div className="py-8 text-center text-white/30 text-sm">No folders found in archive</div>
        ) : (
          nodes.map(node => renderNode(node))
        )}
      </div>
    </div>
  );
}
