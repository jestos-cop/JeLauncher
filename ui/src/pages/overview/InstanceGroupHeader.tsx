import { FolderOpen, Play, Settings2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { type InstanceTab, useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { InstanceAvatar } from '@/components/InstanceAvatar'
import { Button } from '@/components/ui/button'

interface InstanceGroupHeaderProps {
  instance: InstanceSummary
  count: string
  /** Instance page tab that "Edit" opens. */
  editTab: InstanceTab
  onOpenFolder?: () => void
}

/** Instance name and version above its items, with shortcuts to its folder, page and launch. */
export function InstanceGroupHeader({ instance, count, editTab, onOpenFolder }: InstanceGroupHeaderProps) {
  const { t } = useTranslation('common')
  const { navigate } = useNavigation()
  const { session, launch } = useSession(instance.id)
  return (
    <div className="flex items-center gap-2.5">
      <InstanceAvatar instance={instance} className="size-7 rounded-md text-xs" />
      <h2 className="text-sm font-semibold">{instance.name}</h2>
      <span className="font-mono text-xs text-muted-foreground">{instance.minecraftVersion}</span>
      <span className="h-px flex-1 bg-border" />
      <span className="text-xs text-muted-foreground">{count}</span>
      <div className="flex items-center">
        {onOpenFolder && (
          <Button
            size="icon-sm"
            variant="ghost"
            title={t('action.openFolder')}
            aria-label={t('action.openFolder')}
            onClick={onOpenFolder}
          >
            <FolderOpen />
          </Button>
        )}
        <Button
          size="icon-sm"
          variant="ghost"
          title={t('action.edit')}
          aria-label={t('action.edit')}
          onClick={() => navigate({ page: 'instance', instanceId: instance.id, tab: editTab })}
        >
          <Settings2 />
        </Button>
        <Button
          size="icon-sm"
          variant="ghost"
          className="text-primary"
          title={t('action.launch')}
          aria-label={t('action.launch')}
          disabled={session.status !== 'idle'}
          onClick={() => void launch()}
        >
          <Play />
        </Button>
      </div>
    </div>
  )
}
