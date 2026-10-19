import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { commands, events, type MenuAction } from '@/bindings'
import { AboutDialog } from '@/dialogs/AboutDialog'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'
import { NewsDialog } from '@/dialogs/NewsDialog'
import { useNews } from '@/hooks/use-news'
import { unwrap } from '@/lib/command'

type MenuDialog = 'about' | 'newInstance' | 'importInstance' | 'news'

/** Carries out the commands of the native menu bar. */
export function MenuActions() {
  const { navigate } = useNavigation()
  const queryClient = useQueryClient()
  const [dialog, setDialog] = useState<MenuDialog | null>(null)
  const { data: news } = useNews()
  const close = () => setDialog(null)

  useEffect(() => {
    const handle = (action: MenuAction) => {
      switch (action) {
        case 'about':
        case 'newInstance':
        case 'importInstance':
        case 'news':
          return setDialog(action)
        case 'settings':
          return navigate({ page: 'settings', tab: 'launcher' })
        case 'manageAccounts':
          return navigate({ page: 'settings', tab: 'accounts' })
        case 'refresh':
          return void queryClient.invalidateQueries({ queryKey: ['instances'] })
        case 'openInstancesFolder':
          return void unwrap(commands.openLauncherFolder('instances'))
        case 'openLauncherFolder':
          return void unwrap(commands.openLauncherFolder('root'))
        case 'sourceCode':
          return void unwrap(commands.openRepository())
        case 'launcherLog':
          return void unwrap(commands.openLauncherLog())
      }
    }
    const unlisten = events.menuActionEvent.listen(({ payload }) => handle(payload.action))
    return () => void unlisten.then((stop) => stop())
  }, [navigate, queryClient])

  switch (dialog) {
    case 'about':
      return <AboutDialog onClose={close} />
    case 'news':
      return news ? <NewsDialog news={news} onClose={close} /> : null
    case 'newInstance':
    case 'importInstance':
      return (
        <NewInstanceDialog
          open
          initialImport={dialog === 'importInstance' ? '' : undefined}
          onOpenChange={(open) => !open && close()}
        />
      )
    case null:
      return null
  }
}
