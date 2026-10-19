import { useEffect, useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { useLaunch } from '@/app/sessions'
import { commands, events, type CliRequest } from '@/bindings'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'

/** Carries out command line requests: the one the app started with and any sent by later launches. */
export function CliRequests() {
  const { navigate } = useNavigation()
  const launch = useLaunch()
  const [importSource, setImportSource] = useState<string | null>(null)

  useEffect(() => {
    const handle = (request: CliRequest) => {
      if (request.import) setImportSource(request.import)
      if (request.launch) {
        navigate({ page: 'instance', instanceId: request.launch, tab: 'log' })
        void launch(request.launch, { quickPlay: request.quickPlay, demo: false })
      }
    }
    void commands.takeCliRequest().then((result) => {
      if (result.status === 'ok' && result.data) handle(result.data)
    })
    const unlisten = events.cliRequest.listen(({ payload }) => handle(payload))
    return () => void unlisten.then((stop) => stop())
  }, [navigate, launch])

  return (
    importSource && (
      <NewInstanceDialog
        key={importSource}
        open
        initialImport={importSource}
        onOpenChange={(open) => !open && setImportSource(null)}
      />
    )
  )
}
