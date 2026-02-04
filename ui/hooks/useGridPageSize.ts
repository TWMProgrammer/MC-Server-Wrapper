import { useState, useEffect, RefObject } from 'react'

interface GridDimensions {
  cardHeight: number;
  gap: number;
  breakpoints: {
    md: number;
    lg: number;
    xl: number;
  };
}

export function useGridPageSize(
  containerRef: RefObject<HTMLDivElement | null>,
  dimensions: GridDimensions = {
    cardHeight: 224, // h-56
    gap: 16, // gap-4
    breakpoints: {
      md: 768,
      lg: 1024,
      xl: 1280
    }
  }
) {
  const [pageSize, setPageSize] = useState(16)

  useEffect(() => {
    let timeoutId: number | null = null

    const calculatePageSize = () => {
      if (!containerRef.current) return

      const containerHeight = containerRef.current.clientHeight
      
      // Calculate columns based on WINDOW width to match Tailwind breakpoints
      const windowWidth = window.innerWidth
      let columns = 1
      if (windowWidth >= dimensions.breakpoints.xl) columns = 4
      else if (windowWidth >= dimensions.breakpoints.lg) columns = 3
      else if (windowWidth >= dimensions.breakpoints.md) columns = 2

      // Calculate rows that can fit
      const rows = Math.floor((containerHeight + dimensions.gap) / (dimensions.cardHeight + dimensions.gap))
      
      const newPageSize = Math.max(columns, columns * rows)
      setPageSize(newPageSize)
    }

    const debouncedCalculate = () => {
      if (timeoutId) window.clearTimeout(timeoutId)
      timeoutId = window.setTimeout(calculatePageSize, 100)
    }

    const observer = new ResizeObserver(debouncedCalculate)
    if (containerRef.current) {
      observer.observe(containerRef.current)
    }

    calculatePageSize()

    return () => {
      observer.disconnect()
      if (timeoutId) window.clearTimeout(timeoutId)
    }
  }, [containerRef, dimensions.cardHeight, dimensions.gap])

  return pageSize
}
