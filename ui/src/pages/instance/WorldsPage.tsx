import { Archive, Copy, FolderOpen, Play, RefreshCw, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSession } from '@/app/sessions'
import type { World } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { ContextMenuArea } from '@/components/ContextMenuArea'
import { ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu'
import { WorldMenu } from '@/components/WorldMenu'
import { Button } from '@/components/ui/button'
import { openSavesFolder, useBackupWorld, useDeleteWorld, useWorlds } from '@/hooks/use-worlds'

export function WorldsPage({ instanceId }: { instanceId: string }) {
  const { t } = useTranslation(['instance', 'common'])
  const { data: worlds = [], error, isPending, refetch } = useWorlds(instanceId)
  const remove = useDeleteWorld(instanceId)
  const backup = useBackupWorld(instanceId)
  const { session, launch } = useSession(instanceId)
  const [pendingRemoval, setPendingRemoval] = useState<World | null>(null)

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm text-muted-foreground">{t('worlds.count', { count: worlds.length })}</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> {t('common:action.refresh')}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openSavesFolder(instanceId)}>
            <FolderOpen /> {t('common:action.openFolder')}
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {backup.data && (
        <p className="text-sm text-muted-foreground">{t('worlds.backupSaved', { path: backup.data })}</p>
      )}
      {backup.error && <p className="text-sm text-destructive">{backup.error.message}</p>}

      <ContextMenuArea
        menu={
          <ContextMenuContent className="w-max">
            <ContextMenuItem onSelect={() => void refetch()}>
              <RefreshCw /> {t('common:action.refresh')}
            </ContextMenuItem>
            <ContextMenuItem onSelect={() => void openSavesFolder(instanceId)}>
              <FolderOpen /> {t('common:action.openFolder')}
            </ContextMenuItem>
          </ContextMenuContent>
        }
      >
        <div className="min-h-0 flex-1 overflow-auto rounded-md border">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-background text-left text-muted-foreground">
              <tr className="border-b">
                <th className="p-2 font-medium">{t('columns.name')}</th>
                <th className="p-2 font-medium">{t('columns.mode')}</th>
                <th className="p-2 font-medium">{t('columns.version')}</th>
                <th className="p-2 font-medium">{t('columns.lastPlayed')}</th>
                <th className="p-2 font-medium">{t('columns.seed')}</th>
                <th className="w-28 p-2" />
              </tr>
            </thead>
            <tbody>
              {worlds.map((world) => (
                <ContextMenuArea
                  key={world.folder}
                  menu={
                    <WorldMenu
                      instanceId={instanceId}
                      world={world}
                      canPlay={session.status === 'idle'}
                      onPlay={() =>
                        void launch({ quickPlay: { type: 'world', target: world.folder }, demo: false })
                      }
                      onBackUp={() => backup.mutate(world.folder)}
                      onDelete={() => setPendingRemoval(world)}
                    />
                  }
                >
                  <tr className="border-b last:border-0 hover:bg-accent/50">
                    <td className="p-2">
                      <p className="font-medium">{world.name}</p>
                      {world.name !== world.folder && (
                        <p className="text-xs text-muted-foreground">{world.folder}</p>
                      )}
                    </td>
                    <td className="p-2">
                      {t(`worlds.gameMode.${world.hardcore ? 'hardcore' : world.gameMode}`)}
                    </td>
                    <td className="p-2">{world.gameVersion}</td>
                    <td className="p-2">
                      {world.lastPlayed ? new Date(world.lastPlayed).toLocaleString() : '—'}
                    </td>
                    <td className="p-2">
                      {world.seed && (
                        <button
                          type="button"
                          title={t('worlds.copySeed')}
                          className="flex items-center gap-1 font-mono text-xs hover:text-foreground"
                          onClick={() => void navigator.clipboard.writeText(world.seed ?? '')}
                        >
                          {world.seed} <Copy className="size-3" />
                        </button>
                      )}
                    </td>
                    <td className="p-2">
                      <div className="flex gap-2 text-muted-foreground">
                        <button
                          type="button"
                          aria-label={t('item.play', { name: world.name })}
                          title={t('worlds.play')}
                          className="hover:text-foreground disabled:opacity-50"
                          disabled={session.status !== 'idle'}
                          onClick={() =>
                            void launch({ quickPlay: { type: 'world', target: world.folder }, demo: false })
                          }
                        >
                          <Play className="size-4" />
                        </button>
                        <button
                          type="button"
                          aria-label={t('item.backUp', { name: world.name })}
                          title={t('worlds.backUp')}
                          className="hover:text-foreground disabled:opacity-50"
                          disabled={backup.isPending}
                          onClick={() => backup.mutate(world.folder)}
                        >
                          <Archive className="size-4" />
                        </button>
                        <button
                          type="button"
                          aria-label={t('item.delete', { name: world.name })}
                          title={t('common:action.delete')}
                          className="hover:text-destructive"
                          onClick={() => setPendingRemoval(world)}
                        >
                          <Trash2 className="size-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                </ContextMenuArea>
              ))}
            </tbody>
          </table>
          {isPending && <p className="p-3 text-sm text-muted-foreground">{t('worlds.reading')}</p>}
          {!isPending && worlds.length === 0 && (
            <p className="p-3 text-sm text-muted-foreground">{t('worlds.empty')}</p>
          )}
        </div>
      </ContextMenuArea>

      {pendingRemoval && (
        <ConfirmDialog
          title={t('confirm.delete', { name: pendingRemoval.name })}
          description={t('worlds.deleteDescription')}
          confirmLabel={t('common:action.delete')}
          onConfirm={() => remove.mutate(pendingRemoval.folder, { onSuccess: () => setPendingRemoval(null) })}
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </div>
  )
}
