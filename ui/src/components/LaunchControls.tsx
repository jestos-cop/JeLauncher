import { Play, Square, X } from 'lucide-react'
import { useSession } from '@/app/sessions'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { describeProgress, progressPercent } from '@/lib/format'

export function LaunchControls({ instanceId }: { instanceId: string }) {
  const { session, launch, kill } = useSession(instanceId)
  const percent = progressPercent(session.progress)

  return (
    <div className="flex flex-col gap-2">
      {session.status === 'idle' && (
        <Button onClick={() => void launch()}>
          <Play /> Launch
        </Button>
      )}
      {session.status === 'preparing' && (
        <>
          <p className="text-sm text-muted-foreground">{describeProgress(session.progress)}</p>
          {percent !== null && <Progress value={percent} />}
          <Button variant="outline" onClick={() => void kill()}>
            <X /> Cancel
          </Button>
        </>
      )}
      {session.status === 'running' && (
        <Button variant="destructive" onClick={() => void kill()}>
          <Square /> Kill
        </Button>
      )}
      {session.error && <p className="text-sm text-destructive">{session.error}</p>}
    </div>
  )
}
