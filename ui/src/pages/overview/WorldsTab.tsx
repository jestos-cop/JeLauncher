import { useQueries } from '@tanstack/react-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Archive, Globe, Play, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSession } from '@/app/sessions'
import { commands, type InstanceSummary, type World } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { ContextMenuArea } from '@/components/ContextMenuArea'
import { WorldMenu } from '@/components/WorldMenu'
import { Button } from '@/components/ui/button'
import { useInstances } from '@/hooks/use-instances'
import { useStoredState } from '@/hooks/use-stored-state'
import { openSavesFolder, useBackupWorld, useDeleteWorld, worldsKey } from '@/hooks/use-worlds'
import { unwrap } from '@/lib/command'
import { InstanceGroupHeader } from '@/pages/overview/InstanceGroupHeader'
import { OverviewLayout } from '@/pages/overview/OverviewLayout'
import { type OverviewSort, sortGroups } from '@/pages/overview/sort'

/** Worlds of every instance, grouped by instance. */
export function WorldsTab() {
  const { t } = useTranslation('main')
  const { data: instances = [] } = useInstances()
  const results = useQueries({
    queries: instances.map((instance) => ({
      queryKey: worldsKey(instance.id),
      queryFn: () => unwrap(commands.listWorlds(instance.id)),
    })),
  })
  const [sort, setSort] = useStoredState<OverviewSort>('worlds-sort', 'lastPlayed')
  const groups = sortGroups(
    instances
      .map((instance, index) => ({ instance, items: results[index]?.data ?? [] }))
      .filter((group) => group.items.length > 0),
    sort,
    { name: (world) => world.name, recency: (world) => world.lastPlayed },
    (group) => Math.max(...group.items.map((world) => world.lastPlayed)),
  )
  const worldCount = groups.reduce((count, group) => count + group.items.length, 0)
  const isPending = results.some((result) => result.isPending)

  return (
    <OverviewLayout
      tab="worlds"
      summary={t('worlds.summary', { worlds: worldCount, instances: groups.length })}
      sort={sort}
      onSortChange={setSort}
      onRefresh={() => results.forEach((result) => void result.refetch())}
    >
      {isPending && <p className="text-sm text-muted-foreground">{t('worlds.reading')}</p>}
      {!isPending && groups.length === 0 && (
        <p className="text-sm text-muted-foreground">{t('worlds.empty')}</p>
      )}
      {groups.map(({ instance, items }) => (
        <InstanceWorlds key={instance.id} instance={instance} worlds={items} />
      ))}
    </OverviewLayout>
  )
}

function InstanceWorlds({ instance, worlds }: { instance: InstanceSummary; worlds: World[] }) {
  const { t } = useTranslation(['instance', 'common'])
  const { session, launch } = useSession(instance.id)
  const backup = useBackupWorld(instance.id)
  const remove = useDeleteWorld(instance.id)
  const [pendingRemoval, setPendingRemoval] = useState<World | null>(null)

  return (
    <section className="flex flex-col gap-2">
      <InstanceGroupHeader
        instance={instance}
        count={t('worlds.count', { count: worlds.length })}
        editTab="worlds"
        onOpenFolder={() => void openSavesFolder(instance.id)}
      />
      {backup.data && (
        <p className="text-xs text-muted-foreground">{t('worlds.backupSaved', { path: backup.data })}</p>
      )}
      {(backup.error ?? remove.error) && (
        <p className="text-xs text-destructive">{(backup.error ?? remove.error)?.message}</p>
      )}
      <ul className="flex flex-col gap-2">
        {worlds.map((world) => (
          <ContextMenuArea
            key={world.folder}
            menu={
              <WorldMenu
                instanceId={instance.id}
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
            <li className="flex items-center gap-3 rounded-xl border bg-card p-2.5">
              <WorldIcon world={world} />
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-semibold">{world.name}</p>
                <p className="truncate text-xs text-muted-foreground">
                  {[
                    world.gameVersion,
                    t(`worlds.gameMode.${world.hardcore ? 'hardcore' : world.gameMode}`),
                    world.lastPlayed ? new Date(world.lastPlayed).toLocaleDateString() : null,
                  ]
                    .filter(Boolean)
                    .join(' · ')}
                </p>
              </div>
              <div className="flex items-center gap-1">
                <Button
                  size="icon"
                  variant="ghost"
                  className="text-primary"
                  title={t('worlds.play')}
                  aria-label={t('item.play', { name: world.name })}
                  disabled={session.status !== 'idle'}
                  onClick={() =>
                    void launch({ quickPlay: { type: 'world', target: world.folder }, demo: false })
                  }
                >
                  <Play />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  title={t('worlds.backUp')}
                  aria-label={t('item.backUp', { name: world.name })}
                  disabled={backup.isPending}
                  onClick={() => backup.mutate(world.folder)}
                >
                  <Archive />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  className="text-destructive hover:text-destructive"
                  title={t('common:action.delete')}
                  aria-label={t('item.delete', { name: world.name })}
                  onClick={() => setPendingRemoval(world)}
                >
                  <Trash2 />
                </Button>
              </div>
            </li>
          </ContextMenuArea>
        ))}
      </ul>
      {pendingRemoval && (
        <ConfirmDialog
          title={t('confirm.delete', { name: pendingRemoval.name })}
          description={t('worlds.deleteDescription')}
          confirmLabel={t('common:action.delete')}
          onConfirm={() => remove.mutate(pendingRemoval.folder, { onSuccess: () => setPendingRemoval(null) })}
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </section>
  )
}

function WorldIcon({ world }: { world: World }) {
  return (
    <span className="flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted text-muted-foreground">
      {world.icon ? (
        <img
          src={convertFileSrc(world.icon)}
          alt=""
          className="size-full object-cover [image-rendering:pixelated]"
        />
      ) : (
        <Globe className="size-5" />
      )}
    </span>
  )
}
