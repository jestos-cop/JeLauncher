import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { events } from '@/bindings'

/** Reloads the instance list when instance folders change on disk. */
export function InstanceListSync() {
  const queryClient = useQueryClient()
  useEffect(() => {
    const unlisten = events.instancesChangedEvent.listen(
      () => void queryClient.invalidateQueries({ queryKey: ['instances'] }),
    )
    return () => void unlisten.then((stop) => stop())
  }, [queryClient])
  return null
}
