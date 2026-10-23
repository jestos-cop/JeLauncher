import { ArrowUp, Copy, Pencil, Play, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { Server } from '@/bindings'
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu'

interface ServerMenuProps {
  server: Server
  canJoin: boolean
  onJoin: () => void
  onEdit: () => void
  /** Offered only where the list keeps the game's order. */
  onMoveUp?: () => void
  onRemove: () => void
}

/** Right click menu of a saved server. */
export function ServerMenu({ server, canJoin, onJoin, onEdit, onMoveUp, onRemove }: ServerMenuProps) {
  const { t } = useTranslation(['instance', 'common'])
  return (
    <ContextMenuContent className="w-max">
      <ContextMenuItem disabled={!canJoin} onSelect={onJoin}>
        <Play /> {t('servers.join')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => void navigator.clipboard.writeText(server.address)}>
        <Copy /> {t('servers.copyAddress')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onEdit}>
        <Pencil /> {t('common:action.edit')}
      </ContextMenuItem>
      {onMoveUp && (
        <ContextMenuItem onSelect={onMoveUp}>
          <ArrowUp /> {t('servers.moveUp')}
        </ContextMenuItem>
      )}
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" onSelect={onRemove}>
        <Trash2 /> {t('common:action.remove')}
      </ContextMenuItem>
    </ContextMenuContent>
  )
}
