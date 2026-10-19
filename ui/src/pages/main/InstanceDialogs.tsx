import { useTranslation } from 'react-i18next'
import type { InstanceSummary } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PromptDialog } from '@/components/PromptDialog'
import { CopyInstanceDialog } from '@/dialogs/CopyInstanceDialog'
import { ExportInstanceDialog } from '@/dialogs/ExportInstanceDialog'
import { IconPickerDialog } from '@/dialogs/IconPickerDialog'
import { ShortcutDialog } from '@/dialogs/ShortcutDialog'
import { useDeleteInstance, useRenameInstance, useSetInstanceGroup } from '@/hooks/use-instance-actions'

export type InstanceDialog = 'rename' | 'group' | 'icon' | 'copy' | 'export' | 'shortcut' | 'delete'

interface InstanceDialogsProps {
  instance: InstanceSummary
  dialog: InstanceDialog | null
  onClose: () => void
}

/** The dialogs behind the instance actions, shared by the context menu and the side panel. */
export function InstanceDialogs({ instance, dialog, onClose }: InstanceDialogsProps) {
  const { t } = useTranslation(['main', 'common'])
  const rename = useRenameInstance()
  const setGroup = useSetInstanceGroup()
  const remove = useDeleteInstance()

  switch (dialog) {
    case 'rename':
      return (
        <PromptDialog
          title={t('dialogs.renameTitle')}
          initialValue={instance.name}
          confirmLabel={t('common:action.rename')}
          error={rename.error?.message}
          onConfirm={(name) => rename.mutate({ id: instance.id, name }, { onSuccess: onClose })}
          onClose={onClose}
        />
      )
    case 'group':
      return (
        <PromptDialog
          title={t('actions.changeGroup')}
          initialValue={instance.group ?? ''}
          placeholder={t('dialogs.noGroup')}
          confirmLabel={t('common:action.save')}
          allowEmpty
          error={setGroup.error?.message}
          onConfirm={(group) =>
            setGroup.mutate({ id: instance.id, group: group || null }, { onSuccess: onClose })
          }
          onClose={onClose}
        />
      )
    case 'icon':
      return <IconPickerDialog instance={instance} onClose={onClose} />
    case 'copy':
      return <CopyInstanceDialog instance={instance} onClose={onClose} />
    case 'export':
      return <ExportInstanceDialog instance={instance} onClose={onClose} />
    case 'shortcut':
      return <ShortcutDialog instance={instance} onClose={onClose} />
    case 'delete':
      return (
        <ConfirmDialog
          title={t('dialogs.deleteTitle', { name: instance.name })}
          description={t('dialogs.deleteDescription')}
          confirmLabel={t('common:action.delete')}
          onConfirm={() => remove.mutate(instance.id, { onSuccess: onClose })}
          onClose={onClose}
        />
      )
    case null:
      return null
  }
}
