import { Box, Play, ScrollText, Square } from 'lucide-react'
import { useState } from 'react'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { LogDialog } from '@/dialogs/LogDialog'
import { describeProgress, formatPlayTime, progressPercent } from '@/lib/format'

export function InstancePanel({ instance }: { instance: InstanceSummary }) {
  const { session, launch, kill } = useSession(instance.id)
  const percent = progressPercent(session.progress)
  const [isLogOpen, setIsLogOpen] = useState(false)

  return (
    <aside className="flex w-60 shrink-0 flex-col gap-4 border-l p-4">
      <div className="flex flex-col items-center gap-2 text-center">
        <Box className="size-16 text-muted-foreground" />
        <h2 className="font-semibold">{instance.name}</h2>
        <p className="text-sm text-muted-foreground">Minecraft {instance.minecraftVersion}</p>
        <p className="text-xs text-muted-foreground">
          Played {formatPlayTime(instance.totalTimePlayed)}
        </p>
      </div>

      {session.status === 'idle' && (
        <Button onClick={() => void launch()}>
          <Play /> Launch
        </Button>
      )}
      {session.status === 'preparing' && (
        <div className="flex flex-col gap-2">
          <p className="text-sm text-muted-foreground">{describeProgress(session.progress)}</p>
          {percent !== null && <Progress value={percent} />}
        </div>
      )}
      {session.status === 'running' && (
        <Button variant="destructive" onClick={() => void kill()}>
          <Square /> Kill
        </Button>
      )}
      {session.error && <p className="text-sm text-destructive">{session.error}</p>}
      <Button variant="ghost" onClick={() => setIsLogOpen(true)}>
        <ScrollText /> Log
      </Button>

      <LogDialog
        instanceId={instance.id}
        instanceName={instance.name}
        open={isLogOpen}
        onOpenChange={setIsLogOpen}
      />
    </aside>
  )
}
