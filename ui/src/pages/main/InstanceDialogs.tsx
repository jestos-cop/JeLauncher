import type { InstanceSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PromptDialog } from '@/components/PromptDialog'
import { CopyInstanceDialog } from '@/dialogs/CopyInstanceDialog'
import { ExportInstanceDialog } from '@/dialogs/ExportInstanceDialog'
import { useDeleteInstance, useRenameInstance, useSetInstanceGroup } from '@/hooks/use-instance-actions'

export type InstanceDialog = 'rename' | 'group' | 'copy' | 'export' | 'delete'

interface InstanceDialogsProps {
  instance: InstanceSummary
  dialog: InstanceDialog | null
  onClose: () => void
}

/** The dialogs behind the instance actions, shared by the context menu and the side panel. */
export function InstanceDialogs({ instance, dialog, onClose }: InstanceDialogsProps) {
  const rename = useRenameInstance()
  const setGroup = useSetInstanceGroup()
  const remove = useDeleteInstance()

  switch (dialog) {
    case 'rename':
      return (
        <PromptDialog
          title="Rename Instance"
          initialValue={instance.name}
          confirmLabel="Rename"
          error={rename.error?.message}
          onConfirm={(name) => rename.mutate({ id: instance.id, name }, { onSuccess: onClose })}
          onClose={onClose}
        />
      )
    case 'group':
      return (
        <PromptDialog
          title="Change Group"
          initialValue={instance.group ?? ''}
          placeholder="No group"
          confirmLabel="Save"
          allowEmpty
          error={setGroup.error?.message}
          onConfirm={(group) =>
            setGroup.mutate({ id: instance.id, group: group || null }, { onSuccess: onClose })
          }
          onClose={onClose}
        />
      )
    case 'copy':
      return <CopyInstanceDialog instance={instance} onClose={onClose} />
    case 'export':
      return <ExportInstanceDialog instance={instance} onClose={onClose} />
    case 'delete':
      return (
        <ConfirmDialog
          title={`Delete ${instance.name}?`}
          description="The instance folder, including worlds and mods, will be permanently removed."
          confirmLabel="Delete"
          onConfirm={() => remove.mutate(instance.id, { onSuccess: onClose })}
          onClose={onClose}
        />
      )
    case null:
      return null
  }
}
