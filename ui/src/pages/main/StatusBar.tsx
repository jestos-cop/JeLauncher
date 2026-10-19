import { useActiveInstanceIds } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { formatPlayTime } from '@/lib/format'

export function StatusBar({ instances }: { instances: InstanceSummary[] }) {
  const activeIds = useActiveInstanceIds()
  const running = instances.filter((instance) => activeIds.includes(instance.id))
  const totalPlayTime = instances.reduce((total, instance) => total + instance.totalTimePlayed, 0)

  return (
    <footer className="flex items-center gap-4 border-t px-3 py-1 text-xs text-muted-foreground">
      <span>{instances.length} instances</span>
      <span>Total play time: {formatPlayTime(totalPlayTime)}</span>
      {running.length > 0 && (
        <span className="ml-auto truncate">
          Running: {running.map((instance) => instance.name).join(', ')}
        </span>
      )}
    </footer>
  )
}
