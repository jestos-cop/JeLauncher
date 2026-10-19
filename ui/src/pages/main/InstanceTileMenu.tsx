import {
  Copy,
  FolderOpen,
  Image,
  Link,
  Package,
  Pencil,
  Gauge,
  Play,
  Settings2,
  Square,
  Tags,
  Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
} from '@/components/ui/context-menu'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import { type InstanceDialog, InstanceDialogs } from '@/pages/main/InstanceDialogs'

export function InstanceTileMenu({ instance }: { instance: InstanceSummary }) {
  const { t } = useTranslation(['main', 'common'])
  const { session, launch, kill } = useSession(instance.id)
  const { navigate } = useNavigation()
  const [dialog, setDialog] = useState<InstanceDialog | null>(null)
  const isActive = session.status !== 'idle'

  return (
    <>
      <ContextMenuContent className="w-48">
        {isActive ? (
          <ContextMenuItem onSelect={() => void kill()}>
            <Square /> {session.status === 'preparing' ? t('actions.cancelLaunch') : t('common:action.kill')}
          </ContextMenuItem>
        ) : (
          <>
            <ContextMenuItem onSelect={() => void launch()}>
              <Play /> {t('common:action.launch')}
            </ContextMenuItem>
            <ContextMenuItem onSelect={() => void launch({ demo: true })}>
              <Play /> {t('actions.launchDemo')}
            </ContextMenuItem>
            <ContextMenuSub>
              <ContextMenuSubTrigger>
                <Gauge /> {t('actions.launchWithProfiler')}
              </ContextMenuSubTrigger>
              <ContextMenuSubContent>
                <ContextMenuItem onSelect={() => void launch({ profiler: 'jprofiler' })}>
                  JProfiler
                </ContextMenuItem>
                <ContextMenuItem onSelect={() => void launch({ profiler: 'visualVm' })}>
                  VisualVM
                </ContextMenuItem>
              </ContextMenuSubContent>
            </ContextMenuSub>
          </>
        )}
        <ContextMenuItem onSelect={() => navigate({ page: 'instance', instanceId: instance.id, tab: 'log' })}>
          <Settings2 /> {t('common:action.edit')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => void openInstanceFolder(instance.id)}>
          <FolderOpen /> {t('common:action.openFolder')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('rename')}>
          <Pencil /> {t('common:action.rename')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('group')}>
          <Tags /> {t('actions.changeGroup')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('icon')}>
          <Image /> {t('actions.changeIcon')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('copy')}>
          <Copy /> {t('actions.copy')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('export')}>
          <Package /> {t('actions.export')}
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('shortcut')}>
          <Link /> {t('actions.createShortcut')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive" disabled={isActive} onSelect={() => setDialog('delete')}>
          <Trash2 /> {t('common:action.delete')}
        </ContextMenuItem>
      </ContextMenuContent>
      <InstanceDialogs instance={instance} dialog={dialog} onClose={() => setDialog(null)} />
    </>
  )
}
