import { Archive, Copy, FolderOpen, Play, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { World } from '@/bindings'
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu'
import { openSavesFolder } from '@/hooks/use-worlds'

interface WorldMenuProps {
  instanceId: string
  world: World
  canPlay: boolean
  onPlay: () => void
  onBackUp: () => void
  onDelete: () => void
}

/** Right click menu of a world. */
export function WorldMenu({ instanceId, world, canPlay, onPlay, onBackUp, onDelete }: WorldMenuProps) {
  const { t } = useTranslation(['instance', 'common'])
  return (
    <ContextMenuContent className="w-max">
      <ContextMenuItem disabled={!canPlay} onSelect={onPlay}>
        <Play /> {t('worlds.play')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onBackUp}>
        <Archive /> {t('worlds.backUp')}
      </ContextMenuItem>
      {world.seed && (
        <ContextMenuItem onSelect={() => void navigator.clipboard.writeText(world.seed ?? '')}>
          <Copy /> {t('worlds.copySeed')}
        </ContextMenuItem>
      )}
      <ContextMenuItem onSelect={() => void openSavesFolder(instanceId)}>
        <FolderOpen /> {t('common:action.openFolder')}
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" onSelect={onDelete}>
        <Trash2 /> {t('common:action.delete')}
      </ContextMenuItem>
    </ContextMenuContent>
  )
}
