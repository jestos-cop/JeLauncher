import { Play, Square, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useSession } from '@/app/sessions'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { describeProgress, progressPercent } from '@/lib/format'

export function LaunchControls({ instanceId }: { instanceId: string }) {
  const { t } = useTranslation(['launch', 'common'])
  const { session, launch, kill } = useSession(instanceId)
  const percent = progressPercent(session.progress)

  return (
    <div className="flex flex-col gap-2">
      {session.status === 'idle' && (
        <Button onClick={() => void launch()}>
          <Play /> {t('common:action.launch')}
        </Button>
      )}
      {session.status === 'preparing' && (
        <>
          <p className="text-sm text-muted-foreground">{describeProgress(session.progress)}</p>
          {percent !== null && <Progress value={percent} />}
          <Button variant="outline" onClick={() => void kill()}>
            <X /> {t('common:action.cancel')}
          </Button>
        </>
      )}
      {session.status === 'running' && (
        <Button variant="destructive" onClick={() => void kill()}>
          <Square /> {t('common:action.kill')}
        </Button>
      )}
      {session.error && <p className="text-sm text-destructive">{session.error}</p>}
    </div>
  )
}
