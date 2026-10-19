import { FolderOpen, Package, Pencil, Play, Settings2, Square, Tags, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PromptDialog } from '@/components/PromptDialog'
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu'
import { ExportInstanceDialog } from '@/dialogs/ExportInstanceDialog'
import {
  openInstanceFolder,
  useDeleteInstance,
  useRenameInstance,
  useSetInstanceGroup,
} from '@/hooks/use-instance-actions'

type OpenDialog = 'rename' | 'group' | 'export' | 'delete' | null

export function InstanceTileMenu({ instance }: { instance: InstanceSummary }) {
  const { session, launch, kill } = useSession(instance.id)
  const { navigate } = useNavigation()
  const [dialog, setDialog] = useState<OpenDialog>(null)
  const rename = useRenameInstance()
  const setGroup = useSetInstanceGroup()
  const remove = useDeleteInstance()
  const isActive = session.status !== 'idle'
  const close = () => setDialog(null)

  return (
    <>
      <ContextMenuContent className="w-48">
        {isActive ? (
          <ContextMenuItem disabled={session.status !== 'running'} onSelect={() => void kill()}>
            <Square /> Kill
          </ContextMenuItem>
        ) : (
          <ContextMenuItem onSelect={() => void launch()}>
            <Play /> Launch
          </ContextMenuItem>
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
        <ContextMenuItem onSelect={() => setDialog('export')}>
          <Package /> Export…
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive" disabled={isActive} onSelect={() => setDialog('delete')}>
          <Trash2 /> Delete
        </ContextMenuItem>
      </ContextMenuContent>

      {dialog === 'rename' && (
        <PromptDialog
          title="Rename Instance"
          initialValue={instance.name}
          confirmLabel="Rename"
          error={rename.error?.message}
          onConfirm={(name) => rename.mutate({ id: instance.id, name }, { onSuccess: close })}
          onClose={close}
        />
      )}
      {dialog === 'group' && (
        <PromptDialog
          title="Change Group"
          initialValue={instance.group ?? ''}
          placeholder="No group"
          confirmLabel="Save"
          allowEmpty
          error={setGroup.error?.message}
          onConfirm={(group) =>
            setGroup.mutate({ id: instance.id, group: group || null }, { onSuccess: close })
          }
          onClose={close}
        />
      )}
      {dialog === 'export' && <ExportInstanceDialog instance={instance} onClose={close} />}
      {dialog === 'delete' && (
        <ConfirmDialog
          title={`Delete ${instance.name}?`}
          description="The instance folder, including worlds and mods, will be permanently removed."
          confirmLabel="Delete"
          onConfirm={() => remove.mutate(instance.id, { onSuccess: close })}
          onClose={close}
        />
      )}
    </>
  )
}
