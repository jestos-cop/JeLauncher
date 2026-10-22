import { useQueries } from '@tanstack/react-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Archive, ArrowLeft, Globe, Play, RefreshCw, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import { commands, type InstanceSummary, type World } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { InstanceAvatar } from '@/components/InstanceAvatar'
import { Button } from '@/components/ui/button'
import { useInstances } from '@/hooks/use-instances'
import { useBackupWorld, useDeleteWorld, worldsKey } from '@/hooks/use-worlds'
import { unwrap } from '@/lib/command'

/** Worlds of every instance, grouped by instance. */
export function WorldsOverviewPage() {
  const { t } = useTranslation(['main', 'common'])
  const { navigate } = useNavigation()
  const { data: instances = [] } = useInstances()
  const results = useQueries({
    queries: instances.map((instance) => ({
      queryKey: worldsKey(instance.id),
      queryFn: () => unwrap(commands.listWorlds(instance.id)),
    })),
  })
  const groups = instances
    .map((instance, index) => ({ instance, worlds: results[index]?.data ?? [] }))
    .filter((group) => group.worlds.length > 0)
  const worldCount = groups.reduce((count, group) => count + group.worlds.length, 0)
  const isPending = results.some((result) => result.isPending)

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-3 border-b px-3 py-2">
        <Button size="sm" variant="ghost" onClick={() => navigate({ page: 'main' })}>
          <ArrowLeft /> {t('common:action.back')}
        </Button>
        <h1 className="font-semibold">{t('worlds.title')}</h1>
        <span className="text-sm text-muted-foreground">
          {t('worlds.summary', { worlds: worldCount, instances: groups.length })}
        </span>
        <Button
          size="sm"
          variant="ghost"
          className="ml-auto"
          onClick={() => results.forEach((result) => void result.refetch())}
        >
          <RefreshCw /> {t('common:action.refresh')}
        </Button>
      </header>
      <main className="flex min-h-0 flex-1 flex-col gap-5 overflow-auto p-4">
        {isPending && <p className="text-sm text-muted-foreground">{t('worlds.reading')}</p>}
        {!isPending && groups.length === 0 && (
          <p className="text-sm text-muted-foreground">{t('worlds.empty')}</p>
        )}
        {groups.map(({ instance, worlds }) => (
          <InstanceWorlds key={instance.id} instance={instance} worlds={worlds} />
        ))}
      </main>
    </div>
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
      <div className="flex items-center gap-2.5">
        <InstanceAvatar instance={instance} className="size-7 rounded-md text-xs" />
        <h2 className="text-sm font-semibold">{instance.name}</h2>
        <span className="font-mono text-xs text-muted-foreground">{instance.minecraftVersion}</span>
        <span className="h-px flex-1 bg-border" />
        <span className="text-xs text-muted-foreground">{t('worlds.count', { count: worlds.length })}</span>
      </div>
      {backup.data && (
        <p className="text-xs text-muted-foreground">{t('worlds.backupSaved', { path: backup.data })}</p>
      )}
      {(backup.error ?? remove.error) && (
        <p className="text-xs text-destructive">{(backup.error ?? remove.error)?.message}</p>
      )}
      <ul className="flex flex-col gap-2">
        {worlds.map((world) => (
          <li key={world.folder} className="flex items-center gap-3 rounded-xl border bg-card p-2.5">
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
