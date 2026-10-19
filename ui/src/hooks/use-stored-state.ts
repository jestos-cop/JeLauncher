import { useEffect, useState } from 'react'

/** UI preferences that should survive a restart but do not belong in launcher settings. */
export function useStoredState<T>(key: string, initial: T) {
  const [value, setValue] = useState<T>(() => {
    const stored = localStorage.getItem(key)
    if (stored === null) return initial
    try {
      return JSON.parse(stored) as T
    } catch {
      return initial
    }
  })
  useEffect(() => localStorage.setItem(key, JSON.stringify(value)), [key, value])
  return [value, setValue] as const
}
