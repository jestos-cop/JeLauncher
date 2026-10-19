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
  const { session, launch, kill } = useSession(instance.id)
  const { navigate } = useNavigation()
  const [dialog, setDialog] = useState<InstanceDialog | null>(null)
  const isActive = session.status !== 'idle'

  return (
    <>
      <ContextMenuContent className="w-48">
        {isActive ? (
          <ContextMenuItem onSelect={() => void kill()}>
            <Square /> {session.status === 'preparing' ? 'Cancel Launch' : 'Kill'}
          </ContextMenuItem>
        ) : (
          <>
            <ContextMenuItem onSelect={() => void launch()}>
              <Play /> Launch
            </ContextMenuItem>
            <ContextMenuItem onSelect={() => void launch({ demo: true })}>
              <Play /> Launch Demo
            </ContextMenuItem>
            <ContextMenuSub>
              <ContextMenuSubTrigger>
                <Gauge /> Launch with Profiler
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
          <Settings2 /> Edit
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => void openInstanceFolder(instance.id)}>
          <FolderOpen /> Open Folder
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('rename')}>
          <Pencil /> Rename
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('group')}>
          <Tags /> Change Group
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('icon')}>
          <Image /> Change Icon
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('copy')}>
          <Copy /> Copy…
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('export')}>
          <Package /> Export…
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => setDialog('shortcut')}>
          <Link /> Create Shortcut
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive" disabled={isActive} onSelect={() => setDialog('delete')}>
          <Trash2 /> Delete
        </ContextMenuItem>
      </ContextMenuContent>
      <InstanceDialogs instance={instance} dialog={dialog} onClose={() => setDialog(null)} />
    </>
  )
}
