import { Box, Pencil } from 'lucide-react'
import { useNavigation } from '@/app/navigation'
import type { InstanceSummary } from '@/bindings'
import { LaunchControls } from '@/components/LaunchControls'
import { Button } from '@/components/ui/button'
import { formatPlayTime } from '@/lib/format'

export function InstancePanel({ instance }: { instance: InstanceSummary }) {
  const { navigate } = useNavigation()
  const open = () => navigate({ page: 'instance', instanceId: instance.id, tab: 'log' })

  return (
    <aside className="flex w-60 shrink-0 flex-col gap-4 border-l p-4">
      <div className="flex flex-col items-center gap-2 text-center">
        <Box className="size-16 text-muted-foreground" />
        <h2 className="font-semibold">{instance.name}</h2>
        <p className="text-sm text-muted-foreground">
          Minecraft {instance.minecraftVersion}
          {instance.loader && ` · ${instance.loader.kind} ${instance.loader.version}`}
        </p>
        <p className="text-xs text-muted-foreground">
          Played {formatPlayTime(instance.totalTimePlayed)}
        </p>
      </div>

      <LaunchControls instanceId={instance.id} />
      <Button variant="ghost" onClick={open}>
        <Pencil /> Edit
      </Button>
    </aside>
  )
}
