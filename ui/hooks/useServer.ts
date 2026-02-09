import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Instance, ResourceUsage, TransitionType } from '../types'

export function useServer() {
  const [instances, setInstances] = useState<Instance[]>([])
  const [selectedInstanceId, setSelectedInstanceId] = useState<string | null>(null)
  const [status, setStatus] = useState<string>('Stopped')
  const [isTransitioning, setIsTransitioning] = useState<Record<string, TransitionType | null>>({})
  const [usage, setUsage] = useState<ResourceUsage | null>(null)
  const [history, setHistory] = useState<ResourceUsage[]>([])
  const [loading, setLoading] = useState(true)
  const [logs, setLogs] = useState<Record<string, string[]>>({})
  const historyRef = useRef<ResourceUsage[]>([])

  useEffect(() => {
    if (!(window as any).__TAURI_INTERNALS__) {
      setLoading(false);
      return;
    }
    loadInstances()

    const unlisten = listen<{ instance_id: string, line: string }>('server-log', (event) => {
      setLogs(prev => ({
        ...prev,
        [event.payload.instance_id]: [...(prev[event.payload.instance_id] || []), event.payload.line].slice(-500)
      }))
    })

    return () => {
      unlisten.then(f => f())
    }
  }, [])

  useEffect(() => {
    let interval: number;
    interval = window.setInterval(async () => {
      if (!(window as any).__TAURI_INTERNALS__) return;
      try {
        const updatedInstances = await Promise.all(instances.map(async (inst) => {
          const s = await invoke<string>('get_server_status', { instanceId: inst.id })
          return { ...inst, status: s }
        }))

        // Only update if statuses actually changed to avoid unnecessary re-renders
        const hasChanged = updatedInstances.some((inst, idx) => inst.status !== instances[idx].status)
        if (hasChanged) {
          setInstances(updatedInstances)
          
          // Clear transitioning state for instances that reached a stable state
          setIsTransitioning(prev => {
            const next = { ...prev };
            let changed = false;
            updatedInstances.forEach(inst => {
              const type = next[inst.id];
              if (!type) return;

              const isStable = 
                (type === 'starting' && (inst.status === 'Running' || inst.status === 'Crashed')) ||
                (type === 'stopping' && inst.status === 'Stopped') ||
                (type === 'restarting' && inst.status === 'Running');

              if (isStable) {
                delete next[inst.id];
                changed = true;
              }
            });
            return changed ? next : prev;
          });
        }

        if (selectedInstanceId) {
          const currentStatus = updatedInstances.find(i => i.id === selectedInstanceId)?.status || 'Stopped'
          setStatus(currentStatus)

          if (currentStatus === 'Running') {
            const u = await invoke<ResourceUsage>('get_server_usage', { instanceId: selectedInstanceId })
            const usageWithTime = { ...u, timestamp: Date.now() }
            setUsage(u)

            const newHistory = [...historyRef.current, usageWithTime].slice(-100)
            historyRef.current = newHistory
            setHistory(newHistory)
          } else {
            setUsage(null)
          }
        }
      } catch (e) {
        console.error(e)
      }
    }, 2000)
    return () => clearInterval(interval)
  }, [selectedInstanceId, instances])

  async function loadInstances(selectId?: string) {
    if (!(window as any).__TAURI_INTERNALS__) return;
    try {
      const list = await invoke<Instance[]>('list_instances')
      const enrichedList = list.map(inst => {
        let server_type = 'Vanilla';
        if (inst.mod_loader) {
          server_type = inst.mod_loader.charAt(0).toUpperCase() + inst.mod_loader.slice(1);
          if (inst.mod_loader.toLowerCase() === 'neoforge') server_type = 'NeoForge';
        }

        return {
          ...inst,
          server_type: inst.server_type || server_type,
          ip: inst.ip || '127.0.0.1',
          port: inst.port || inst.settings.port || 25565,
          description: inst.description || inst.settings.description || 'There is no description for this server.',
          max_players: inst.max_players || 20
        }
      })
      setInstances(enrichedList)
      if (selectId) {
        setSelectedInstanceId(selectId)
      } else if (enrichedList.length > 0 && selectedInstanceId && !enrichedList.find(i => i.id === selectedInstanceId)) {
        setSelectedInstanceId(null)
      } else if (enrichedList.length === 0) {
        setSelectedInstanceId(null)
      }
      setLoading(false)
    } catch (e) {
      console.error(e)
      setLoading(false)
    }
  }

  async function startServer(instanceId?: string) {
    const id = instanceId || selectedInstanceId;
    if (!id || !(window as any).__TAURI_INTERNALS__) return;
    setIsTransitioning(prev => ({ ...prev, [id]: 'starting' }))
    try {
      await invoke('start_server', { instanceId: id })
    } catch (e) {
      console.error(e)
      setIsTransitioning(prev => {
        const next = { ...prev };
        delete next[id];
        return next;
      })
    }
  }

  async function stopServer(instanceId?: string) {
    const id = instanceId || selectedInstanceId;
    if (!id || !(window as any).__TAURI_INTERNALS__) return;
    setIsTransitioning(prev => ({ ...prev, [id]: 'stopping' }))
    try {
      await invoke('stop_server', { instanceId: id })
    } catch (e) {
      console.error(e)
      setIsTransitioning(prev => {
        const next = { ...prev };
        delete next[id];
        return next;
      })
    }
  }

  async function restartServer(instanceId?: string) {
    const id = instanceId || selectedInstanceId;
    if (!id || !(window as any).__TAURI_INTERNALS__) return;
    setIsTransitioning(prev => ({ ...prev, [id]: 'restarting' }))
    try {
      await invoke('send_command', { instanceId: id, command: 'restart' })
    } catch (e) {
      console.error(e)
      setIsTransitioning(prev => {
        const next = { ...prev };
        delete next[id];
        return next;
      })
    }
  }

  async function sendCommand(command: string) {
    if (!command || !selectedInstanceId) return
    const cmdTrimmed = command.trim().toLowerCase()
    const id = selectedInstanceId
    const isBungee = currentInstance?.mod_loader === 'bungeecord'
    
    if (cmdTrimmed === 'stop' || (cmdTrimmed === 'end' && isBungee)) {
      setIsTransitioning(prev => ({ ...prev, [id]: 'stopping' }))
    } else if (cmdTrimmed === 'restart') {
      setIsTransitioning(prev => ({ ...prev, [id]: 'restarting' }))
    }
    try {
      await invoke('send_command', { instanceId: id, command })
    } catch (e) {
      console.error(e)
      if (cmdTrimmed === 'stop' || cmdTrimmed === 'restart') {
        setIsTransitioning(prev => {
          const next = { ...prev };
          delete next[id];
          return next;
        })
      }
    }
  }

  const currentInstance = instances.find(i => i.id === selectedInstanceId);

  return {
    instances,
    selectedInstanceId,
    setSelectedInstanceId,
    currentInstance,
    status,
    isTransitioning,
    usage,
    history,
    loading,
    logs,
    loadInstances,
    startServer,
    stopServer,
    restartServer,
    sendCommand
  }
}
