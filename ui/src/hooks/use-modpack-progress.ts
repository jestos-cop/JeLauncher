import { useEffect, useState } from 'react'
import { events, type DownloadProgress } from '@/bindings'

/** Download progress of the modpack install or update that is currently running. */
export function useModpackProgress(active: boolean) {
  const [progress, setProgress] = useState<DownloadProgress | null>(null)
  useEffect(() => {
    if (!active) return
    const unlisten = events.modpackProgressEvent.listen(({ payload }) => setProgress(payload.progress))
    return () => {
      void unlisten.then((stop) => stop())
      setProgress(null)
    }
  }, [active])
  return progress
}
