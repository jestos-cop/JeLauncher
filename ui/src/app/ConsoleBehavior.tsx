import { useEffect, useRef } from 'react'
import { useNavigation } from '@/app/navigation'
import { events, type LauncherSettings } from '@/bindings'
import { useSettings } from '@/hooks/use-settings'

/** Opens and closes an instance's log page around launches, as configured in the launcher settings. */
export function ConsoleBehavior() {
  const { view, navigate } = useNavigation()
  const { data: settings } = useSettings()
  const latest = useRef<{ view: typeof view; launcher?: LauncherSettings }>({ view })

  useEffect(() => {
    latest.current = { view, launcher: settings?.launcher }
  }, [view, settings])

  useEffect(() => {
    const showLog = (instanceId: string) => navigate({ page: 'instance', instanceId, tab: 'log' })
    const subscriptions = [
      events.launchProgressEvent.listen(({ payload }) => {
        if (payload.progress.stage === 'starting' && latest.current.launcher?.showConsole) {
          showLog(payload.instanceId)
        }
      }),
      events.instanceExitedEvent.listen(({ payload }) => {
        const { view: current, launcher } = latest.current
        const crashed = payload.exitCode !== null && payload.exitCode !== 0
        const isShowingLog =
          current.page === 'instance' && current.instanceId === payload.instanceId && current.tab === 'log'
        if (crashed && launcher?.showConsoleOnError) showLog(payload.instanceId)
        else if (!crashed && isShowingLog && launcher?.autoCloseConsole) navigate({ page: 'main' })
      }),
    ]
    return () => subscriptions.forEach((subscription) => void subscription.then((unlisten) => unlisten()))
  }, [navigate])

  return null
}
