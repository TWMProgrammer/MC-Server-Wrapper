import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Database,
  Search,
  RefreshCw,
  Table as TableIcon,
  ChevronRight,
  ChevronDown,
  X,
  Download
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { createPortal } from 'react-dom'
import { useAppSettings } from '../hooks/useAppSettings'
import { useToast } from '../hooks/useToast'

interface DatabaseExplorerModalProps {
  instanceId: string;
  onClose: () => void;
}

interface TableData {
  columns: string[];
  rows: any[][];
}

interface DBFile {
  path: string;
  name: string;
  tables: string[];
  isOpen: boolean;
  dbType: 'SQLite' | 'H2' | 'SQL';
}

interface DBGroup {
  name: string;
  files: DBFile[];
  isOpen: boolean;
}

export function DatabaseExplorerModal({ instanceId, onClose }: DatabaseExplorerModalProps) {
  const { settings } = useAppSettings()
  const { showToast } = useToast()
  const [groups, setGroups] = useState<DBGroup[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedDb, setSelectedDb] = useState<string | null>(null)
  const [selectedTable, setSelectedTable] = useState<string | null>(null)
  const [tableData, setTableData] = useState<TableData | null>(null)
  const [dataLoading, setDataLoading] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [sqlContent, setSqlContent] = useState<string | null>(null)
  const [page, setPage] = useState(0)
  const pageSize = 50

  useEffect(() => {
    loadDatabases()
  }, [])

  const loadDatabases = async () => {
    setLoading(true)
    setSqlContent(null)
    try {
      const response = await invoke<any[]>('explore_find_databases', { instanceId })
      const formattedGroups: DBGroup[] = response.map(g => ({
        name: g.name,
        isOpen: response.length === 1 || g.name === 'General Plugins',
        files: g.files.map((f: any) => ({
          path: f.path,
          name: f.name,
          dbType: f.db_type,
          tables: [],
          isOpen: false
        }))
      }))
      setGroups(formattedGroups)
    } catch (err: any) {
      console.error('Failed to find databases:', err)
      const message = typeof err === 'string' ? err : err.message || 'Unknown error'
      showToast(`Failed to find databases: ${message}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  const toggleGroup = (index: number) => {
    const newGroups = [...groups]
    newGroups[index].isOpen = !newGroups[index].isOpen
    setGroups(newGroups)
  }

  const toggleDb = async (groupIndex: number, fileIndex: number) => {
    const group = groups[groupIndex]
    const db = group.files[fileIndex]

    if (db.dbType === 'H2') {
      showToast('H2 databases (.mv.db) are currently only supported for identification. Inspection requires a Java driver.', 'info')
      return
    }

    if (!db.isOpen && db.tables.length === 0) {
      // Don't clear sqlContent here yet, we might want to keep it if it's already selected
      try {
        const tables = await invoke<string[]>('explore_list_tables', { path: db.path })
        const newGroups = [...groups]
        newGroups[groupIndex].files[fileIndex] = { ...db, tables, isOpen: true }
        setGroups(newGroups)
      } catch (err: any) {
        console.error('Failed to list tables:', err)
        const message = typeof err === 'string' ? err : err.message || 'Unknown error'
        showToast(`Failed to list tables: ${message}`, 'error')
      }
    } else {
      const newGroups = [...groups]
      newGroups[groupIndex].files[fileIndex] = { ...db, isOpen: !db.isOpen }
      setGroups(newGroups)
    }
  }

  const handleSourceView = async (dbPath: string) => {
    setSelectedDb(dbPath)
    setSelectedTable(null)
    setTableData(null)
    setDataLoading(true)
    try {
      const content = await invoke<string>('explore_read_sql_file', { path: dbPath })
      setSqlContent(content)
    } catch (err: any) {
      showToast(`Failed to read SQL file: ${err}`, 'error')
    } finally {
      setDataLoading(false)
    }
  }

  const loadTableData = async (dbPath: string, tableName: string) => {
    setSelectedDb(dbPath)
    setSelectedTable(tableName)
    setSqlContent(null)
    setDataLoading(true)
    setPage(0)
    try {
      const data = await invoke<TableData>('explore_get_data', {
        path: dbPath,
        table: tableName,
        limit: pageSize,
        offset: 0
      })
      setTableData(data)
    } catch (err: any) {
      console.error('Failed to load table data:', err)
      const message = typeof err === 'string' ? err : err.message || 'Unknown error'
      showToast(`Failed to load table data: ${message}`, 'error')
    } finally {
      setDataLoading(false)
    }
  }

  const handleNextPage = async () => {
    if (!selectedDb || !selectedTable) return
    const nextOffset = (page + 1) * pageSize
    setDataLoading(true)
    try {
      const data = await invoke<TableData>('explore_get_data', {
        path: selectedDb,
        table: selectedTable,
        limit: pageSize,
        offset: nextOffset
      })
      setTableData(data)
      setPage(page + 1)
    } catch (err) {
      showToast('Failed to load next page', 'error')
    } finally {
      setDataLoading(false)
    }
  }

  const handlePrevPage = async () => {
    if (!selectedDb || !selectedTable || page === 0) return
    const prevOffset = (page - 1) * pageSize
    setDataLoading(true)
    try {
      const data = await invoke<TableData>('explore_get_data', {
        path: selectedDb,
        table: selectedTable,
        limit: pageSize,
        offset: prevOffset
      })
      setTableData(data)
      setPage(page - 1)
    } catch (err) {
      showToast('Failed to load previous page', 'error')
    } finally {
      setDataLoading(false)
    }
  }

  return createPortal(
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center p-4 overflow-hidden"
      style={{
        width: `${100 / settings.scaling}%`,
        height: `${100 / settings.scaling}%`,
        transform: `scale(${settings.scaling})`,
        transformOrigin: 'top left',
      }}
    >
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
        className="relative bg-[#16161a] border border-white/10 rounded-3xl shadow-2xl w-full max-w-6xl h-[85vh] flex flex-col overflow-hidden"
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-white/5 bg-[#1a1a1f]">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-xl">
              <Database className="text-primary" size={24} />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white tracking-tight">Database Explorer</h2>
              <p className="text-sm text-gray-500 font-medium">Browse and inspect server databases</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/5 rounded-xl text-gray-500 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        <div className="flex-1 flex overflow-hidden">
          {/* Sidebar */}
          <div className="w-72 border-r border-white/5 overflow-y-auto p-4 space-y-4 bg-[#0e0e11]">
            <h3 className="text-[10px] font-black text-gray-500 uppercase tracking-[0.2em] px-2 mb-2">Databases</h3>
            {loading ? (
              <div className="flex items-center justify-center py-8">
                <RefreshCw className="animate-spin text-primary opacity-50" size={24} />
              </div>
            ) : groups.length === 0 ? (
              <p className="text-sm text-gray-500 px-2 italic font-medium">No databases found</p>
            ) : (
              groups.map((group, gIdx) => (
                <div key={group.name} className="space-y-1">
                  <button
                    onClick={() => toggleGroup(gIdx)}
                    className="w-full flex items-center gap-2 p-2 rounded-xl text-left hover:bg-white/5 text-gray-400 transition-all group"
                  >
                    <motion.div
                      animate={{ rotate: group.isOpen ? 90 : 0 }}
                      transition={{ duration: 0.2 }}
                    >
                      <ChevronRight size={14} className="text-gray-600 group-hover:text-gray-400" />
                    </motion.div>
                    <span className="text-[11px] font-black uppercase tracking-wider text-gray-500 group-hover:text-gray-300">{group.name}</span>
                  </button>

                  <AnimatePresence>
                    {group.isOpen && (
                      <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        className="overflow-hidden pl-4 space-y-1"
                      >
                        {group.files.map((db, fIdx) => (
                          <div key={db.path} className="space-y-1">
                            <button
                              onClick={() => toggleDb(gIdx, fIdx)}
                              className={`w-full flex items-center gap-2 p-2 rounded-xl text-left transition-all ${selectedDb === db.path ? 'text-primary' : 'hover:bg-white/5 text-gray-400'}`}
                            >
                              <motion.div
                                animate={{ rotate: db.isOpen ? 90 : 0 }}
                                transition={{ duration: 0.2 }}
                              >
                                <ChevronRight size={14} />
                              </motion.div>
                              <span className="text-sm font-bold truncate flex-1">{db.name}</span>
                              <div className="flex items-center gap-2">
                                {db.dbType === 'SQL' && (
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      handleSourceView(db.path);
                                    }}
                                    className="p-1 hover:bg-white/10 rounded transition-all text-gray-500 hover:text-gray-300"
                                    title="View Source SQL"
                                  >
                                    <Download size={12} />
                                  </button>
                                )}
                                <span className={`text-[9px] px-1.5 py-0.5 rounded font-black uppercase tracking-tighter ${db.dbType === 'SQLite' ? 'bg-blue-500/10 text-blue-400' :
                                  db.dbType === 'H2' ? 'bg-orange-500/10 text-orange-400' :
                                    'bg-green-500/10 text-green-400'
                                  }`}>
                                  {db.dbType}
                                </span>
                              </div>
                            </button>

                            <AnimatePresence>
                              {db.isOpen && (
                                <motion.div
                                  initial={{ height: 0, opacity: 0 }}
                                  animate={{ height: 'auto', opacity: 1 }}
                                  exit={{ height: 0, opacity: 0 }}
                                  className="overflow-hidden pl-6 space-y-1"
                                >
                                  {db.tables.length === 0 ? (
                                    <p className="text-[10px] text-gray-600 italic py-1">No tables</p>
                                  ) : db.tables.map(table => (
                                    <button
                                      key={table}
                                      onClick={() => loadTableData(db.path, table)}
                                      className={`w-full flex items-center gap-2 p-2 rounded-lg text-left transition-all ${selectedTable === table && selectedDb === db.path ? 'bg-primary text-white shadow-lg shadow-primary/20' : 'hover:bg-white/5 text-gray-500 hover:text-gray-300'}`}
                                    >
                                      <TableIcon size={14} />
                                      <span className="text-xs font-bold truncate">{table}</span>
                                    </button>
                                  ))}
                                </motion.div>
                              )}
                            </AnimatePresence>
                          </div>
                        ))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              ))
            )}
          </div>

          {/* Main Area */}
          <div className="flex-1 flex flex-col overflow-hidden bg-[#16161a]">
            {sqlContent ? (
              <div className="h-full flex flex-col">
                <div className="p-4 border-b border-white/5 flex items-center justify-between bg-[#1a1a1f]">
                  <div className="flex items-center gap-2 text-sm font-medium">
                    <TableIcon className="w-4 h-4 text-green-400" />
                    <span className="text-white font-bold">{selectedDb?.split(/[\\/]/).pop()}</span>
                  </div>
                </div>
                <div className="flex-1 overflow-auto p-4 bg-[#0e0e11] custom-scrollbar">
                  <pre className="text-xs font-mono text-gray-300 whitespace-pre-wrap leading-relaxed">
                    {sqlContent}
                  </pre>
                </div>
              </div>
            ) : selectedTable ? (
              <>
                {/* Toolbar */}
                <div className="p-4 border-b border-white/5 flex items-center justify-between gap-4 bg-[#1a1a1f]">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="relative flex-1 max-w-md">
                      <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                      <input
                        type="text"
                        placeholder="Search rows..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-10 pr-4 py-2 bg-[#0e0e11] border border-white/5 rounded-xl text-sm font-medium focus:outline-none focus:border-primary/50 transition-colors"
                      />
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => loadTableData(selectedDb!, selectedTable!)}
                      className="p-2 hover:bg-white/5 rounded-xl text-gray-400 transition-colors"
                      title="Refresh data"
                    >
                      <RefreshCw size={18} className={dataLoading ? 'animate-spin' : ''} />
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-white/5 hover:bg-white/10 text-gray-300 rounded-xl text-sm font-bold border border-white/5 transition-all active:scale-95">
                      <Download size={16} />
                      Export
                    </button>
                  </div>
                </div>

                {/* Data Grid */}
                <div className="flex-1 overflow-auto custom-scrollbar bg-[#16161a]">
                  {dataLoading && !tableData ? (
                    <div className="h-full flex items-center justify-center">
                      <RefreshCw className="animate-spin text-primary opacity-50" size={32} />
                    </div>
                  ) : tableData ? (
                    <table className="w-full text-left border-collapse min-w-full">
                      <thead className="sticky top-0 bg-[#1a1a1f] z-10">
                        <tr className="border-b border-white/5">
                          {tableData.columns.map(col => (
                            <th key={col} className="px-4 py-3 text-[10px] font-black text-gray-500 uppercase tracking-widest bg-white/5 whitespace-nowrap">
                              {col}
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-white/5">
                        {tableData.rows
                          .filter(row =>
                            row.some(val =>
                              val?.toString().toLowerCase().includes(searchQuery.toLowerCase())
                            )
                          )
                          .map((row, ridx) => (
                            <tr key={ridx} className="hover:bg-white/[0.02] transition-colors group">
                              {row.map((val, cidx) => (
                                <td key={cidx} className="px-4 py-3 text-xs text-gray-400 font-mono whitespace-nowrap overflow-hidden text-ellipsis max-w-[300px] group-hover:text-gray-200">
                                  {val === null ? (
                                    <span className="text-gray-600 italic text-[10px]">NULL</span>
                                  ) : typeof val === 'object' ? (
                                    JSON.stringify(val)
                                  ) : (
                                    val.toString()
                                  )}
                                </td>
                              ))}
                            </tr>
                          ))}
                        {tableData.rows.length === 0 && (
                          <tr>
                            <td colSpan={tableData.columns.length} className="px-4 py-12 text-center text-gray-500 font-medium italic">
                              Table is empty
                            </td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  ) : null}
                </div>

                {/* Footer / Pagination */}
                <div className="px-6 py-4 border-t border-white/5 flex items-center justify-between bg-[#1a1a1f]">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 bg-primary/10 text-primary rounded text-[10px] font-black uppercase tracking-wider">
                      {selectedTable}
                    </span>
                    <span className="text-[11px] text-gray-500 font-bold">
                      Page {page + 1}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={handlePrevPage}
                      disabled={page === 0 || dataLoading}
                      className="px-4 py-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 text-gray-300 rounded-xl text-xs font-black uppercase tracking-wider border border-white/5 transition-all active:scale-95"
                    >
                      Previous
                    </button>
                    <button
                      onClick={handleNextPage}
                      disabled={!tableData || tableData.rows.length < pageSize || dataLoading}
                      className="px-4 py-2 bg-white/5 hover:bg-white/10 disabled:opacity-30 text-gray-300 rounded-xl text-xs font-black uppercase tracking-wider border border-white/5 transition-all active:scale-95"
                    >
                      Next
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div className="flex-1 flex flex-col items-center justify-center p-8 text-center bg-[#111114]">
                <div className="p-8 bg-white/5 rounded-3xl mb-6 shadow-inner">
                  {selectedDb ? (
                    <TableIcon size={56} className="text-gray-600" />
                  ) : (
                    <Database size={56} className="text-gray-600" />
                  )}
                </div>
                <h3 className="text-2xl font-black text-white mb-2 tracking-tight">
                  {selectedDb ? 'No Table Selected' : 'No Database Selected'}
                </h3>
                <p className="text-gray-500 max-w-xs font-medium">
                  {selectedDb
                    ? 'Select a table from the sidebar to view its contents.'
                    : 'Select a database from the sidebar and click on a table to view its contents.'}
                </p>
              </div>
            )}
          </div>
        </div>
      </motion.div>
    </div>,
    document.body
  )
}
