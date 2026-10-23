import { ExternalLink, History, Lock, LockOpen, Power, PowerOff, Trash2, Unlink } from 'lucide-react'
import type { ComponentType, ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import type { ResourceFile } from '@/bindings'
import { PROVIDER_NAMES } from '@/lib/providers'

/** The item and separator of the menu kind the actions are shown in (dropdown or context menu). */
export interface MenuParts {
  Item: ComponentType<{ onSelect?: () => void; variant?: 'default' | 'destructive'; children: ReactNode }>
  Separator: ComponentType
}

interface ResourceActionsProps {
  parts: MenuParts
  file: ResourceFile
  onToggle: () => void
  onViewSource: () => void
  onChangeVersion: () => void
  onToggleLock: () => void
  onForgetSource: () => void
  onRemove: () => void
}

/** Actions on one installed mod or pack. */
export function ResourceActions({
  parts: { Item, Separator },
  file,
  onToggle,
  onViewSource,
  onChangeVersion,
  onToggleLock,
  onForgetSource,
  onRemove,
}: ResourceActionsProps) {
  const { t } = useTranslation(['instance', 'common'])
  return (
    <>
      <Item onSelect={onToggle}>
        {file.enabled ? <PowerOff /> : <Power />}
        {file.enabled ? t('resources.disable') : t('resources.enable')}
      </Item>
      {file.source && (
        <>
          <Separator />
          <Item onSelect={onViewSource}>
            <ExternalLink />
            {t('resources.viewOn', { provider: PROVIDER_NAMES[file.source.provider] })}
          </Item>
          <Item onSelect={onChangeVersion}>
            <History /> {t('resources.changeVersion')}
          </Item>
          <Item onSelect={onToggleLock}>
            {file.source.lockUpdate ? <LockOpen /> : <Lock />}
            {file.source.lockUpdate ? t('resources.unlockUpdates') : t('resources.lockUpdates')}
          </Item>
          <Item onSelect={onForgetSource}>
            <Unlink /> {t('resources.forgetSource')}
          </Item>
        </>
      )}
      <Separator />
      <Item variant="destructive" onSelect={onRemove}>
        <Trash2 /> {t('common:action.remove')}
      </Item>
    </>
  )
}
