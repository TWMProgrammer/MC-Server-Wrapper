import { useEffect, useRef } from 'react'

export function useConsoleScroll(logs: any, selectedInstanceId: string | null) {
  const consoleEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [logs, selectedInstanceId])

  return consoleEndRef
}
