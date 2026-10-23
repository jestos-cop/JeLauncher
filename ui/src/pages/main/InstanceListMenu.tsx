import { FolderOpen, Globe, Import, Plus, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { commands } from '@/bindings'
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu'
import { unwrap } from '@/lib/command'

interface InstanceListMenuProps {
  onCreate: () => void
  onImport: () => void
  onRefresh: () => void
}

/** Right click menu of the empty space around the instances. */
export function InstanceListMenu({ onCreate, onImport, onRefresh }: InstanceListMenuProps) {
  const { t } = useTranslation(['main', 'common'])
  const { navigate } = useNavigation()
  return (
    <ContextMenuContent className="w-max">
      <ContextMenuItem onSelect={onCreate}>
        <Plus /> {t('toolbar.addInstance')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onImport}>
        <Import /> {t('listMenu.import')}
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onSelect={onRefresh}>
        <RefreshCw /> {t('common:action.refresh')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => navigate({ page: 'overview', tab: 'worlds' })}>
        <Globe /> {t('toolbar.worlds')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => void unwrap(commands.openLauncherFolder('instances'))}>
        <FolderOpen /> {t('listMenu.openInstancesFolder')}
      </ContextMenuItem>
    </ContextMenuContent>
  )
}
