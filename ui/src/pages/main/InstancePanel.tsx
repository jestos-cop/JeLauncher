import { Box, Copy, FolderOpen, Package, Pencil, Settings2, Tags, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { LaunchControls } from '@/components/LaunchControls'
import { Button } from '@/components/ui/button'
import { openInstanceFolder } from '@/hooks/use-instance-actions'
import { formatPlayTime } from '@/lib/format'
import { type InstanceDialog, InstanceDialogs } from '@/pages/main/InstanceDialogs'

export function InstancePanel({ instance }: { instance: InstanceSummary }) {
  const { navigate } = useNavigation()
  const { session } = useSession(instance.id)
  const [dialog, setDialog] = useState<InstanceDialog | null>(null)
  const action = (label: string, icon: React.ReactNode, onClick: () => void, disabled = false) => (
    <Button variant="ghost" size="sm" className="justify-start" disabled={disabled} onClick={onClick}>
      {icon} {label}
    </Button>
  )

  return (
    <aside className="flex w-60 shrink-0 flex-col gap-4 overflow-auto border-l p-4">
      <div className="flex flex-col items-center gap-2 text-center">
        <Box className="size-16 text-muted-foreground" />
        <h2 className="font-semibold">{instance.name}</h2>
        <p className="text-sm text-muted-foreground">
          Minecraft {instance.minecraftVersion}
          {instance.loader && ` · ${instance.loader.kind} ${instance.loader.version}`}
        </p>
        <p className="text-xs text-muted-foreground">Played {formatPlayTime(instance.totalTimePlayed)}</p>
      </div>

      <LaunchControls instanceId={instance.id} />
      <div className="flex flex-col">
        {action('Edit', <Settings2 />, () =>
          navigate({ page: 'instance', instanceId: instance.id, tab: 'log' }),
        )}
        {action('Open Folder', <FolderOpen />, () => void openInstanceFolder(instance.id))}
        {action('Rename', <Pencil />, () => setDialog('rename'))}
        {action('Change Group', <Tags />, () => setDialog('group'))}
        {action('Copy…', <Copy />, () => setDialog('copy'))}
        {action('Export…', <Package />, () => setDialog('export'))}
        {action('Delete', <Trash2 />, () => setDialog('delete'), session.status !== 'idle')}
      </div>
      <InstanceDialogs instance={instance} dialog={dialog} onClose={() => setDialog(null)} />
    </aside>
  )
}
