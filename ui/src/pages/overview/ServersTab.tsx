import { useQueries, useQueryClient } from '@tanstack/react-query'
import { Play, Server as ServerIcon, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import { commands, type InstanceSummary, type Server } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { ContextMenuArea } from '@/components/ContextMenuArea'
import { ServerMenu } from '@/components/ServerMenu'
import { Button } from '@/components/ui/button'
import { useInstances } from '@/hooks/use-instances'
import { useStoredState } from '@/hooks/use-stored-state'
import { serversKey, useRemoveServer, useServerStatus } from '@/hooks/use-servers'
import { unwrap } from '@/lib/command'
import { InstanceGroupHeader } from '@/pages/overview/InstanceGroupHeader'
import { OverviewLayout } from '@/pages/overview/OverviewLayout'
import { type OverviewSort, sortGroups } from '@/pages/overview/sort'

/** A saved server with its place in the instance's list, which edits refer to. */
interface ListedServer {
  index: number
  server: Server
}

/** Saved servers of every instance, grouped by instance. */
export function ServersTab() {
  const { t } = useTranslation('main')
  const queryClient = useQueryClient()
  const { data: instances = [] } = useInstances()
  const results = useQueries({
    queries: instances.map((instance) => ({
      queryKey: serversKey(instance.id),
      queryFn: () => unwrap(commands.listServers(instance.id)),
    })),
  })
  const [sort, setSort] = useStoredState<OverviewSort>('servers-sort', 'lastPlayed')
  const groups = sortGroups(
    instances
      .map((instance, index) => ({
        instance,
        items: (results[index]?.data ?? []).map((server, position) => ({ index: position, server })),
      }))
      .filter((group) => group.items.length > 0),
    sort,
    { name: (listed: ListedServer) => listed.server.name, recency: (listed: ListedServer) => -listed.index },
    (group) => group.instance.lastLaunchTime,
  )
  const serverCount = groups.reduce((count, group) => count + group.items.length, 0)
  const isPending = results.some((result) => result.isPending)

  return (
    <OverviewLayout
      tab="servers"
      summary={t('servers.summary', { servers: serverCount, instances: groups.length })}
      sort={sort}
      onSortChange={setSort}
      onRefresh={() => {
        results.forEach((result) => void result.refetch())
        void queryClient.invalidateQueries({ queryKey: ['server-status'] })
      }}
    >
      {isPending && <p className="text-sm text-muted-foreground">{t('servers.reading')}</p>}
      {!isPending && groups.length === 0 && (
        <p className="text-sm text-muted-foreground">{t('servers.empty')}</p>
      )}
      {groups.map(({ instance, items }) => (
        <InstanceServers key={instance.id} instance={instance} servers={items} />
      ))}
    </OverviewLayout>
  )
}

function InstanceServers({ instance, servers }: { instance: InstanceSummary; servers: ListedServer[] }) {
  const { t } = useTranslation(['instance', 'common'])
  const { navigate } = useNavigation()
  const { session, launch } = useSession(instance.id)
  const remove = useRemoveServer(instance.id)
  const [pendingRemoval, setPendingRemoval] = useState<ListedServer | null>(null)

  return (
    <section className="flex flex-col gap-2">
      <InstanceGroupHeader
        instance={instance}
        count={t('servers.count', { count: servers.length })}
        editTab="servers"
      />
      {remove.error && <p className="text-xs text-destructive">{remove.error.message}</p>}
      <ul className="flex flex-col gap-2">
        {servers.map((listed) => (
          <ServerCard
            key={`${listed.index}-${listed.server.address}`}
            server={listed.server}
            canJoin={session.status === 'idle'}
            onJoin={() =>
              void launch({ quickPlay: { type: 'server', target: listed.server.address }, demo: false })
            }
            onEdit={() => navigate({ page: 'instance', instanceId: instance.id, tab: 'servers' })}
            onRemove={() => setPendingRemoval(listed)}
          />
        ))}
      </ul>
      {pendingRemoval && (
        <ConfirmDialog
          title={t('confirm.remove', { name: pendingRemoval.server.name || t('servers.unnamed') })}
          description={t('servers.removeDescription')}
          confirmLabel={t('common:action.remove')}
          onConfirm={() => remove.mutate(pendingRemoval.index, { onSuccess: () => setPendingRemoval(null) })}
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </section>
  )
}

interface ServerCardProps {
  server: Server
  canJoin: boolean
  onJoin: () => void
  onEdit: () => void
  onRemove: () => void
}

function ServerCard({ server, canJoin, onJoin, onEdit, onRemove }: ServerCardProps) {
  const { t } = useTranslation(['instance', 'common'])
  const status = useServerStatus(server.address)
  const icon = status.data?.favicon ?? (server.icon && `data:image/png;base64,${server.icon}`)

  return (
    <ContextMenuArea
      menu={
        <ServerMenu server={server} canJoin={canJoin} onJoin={onJoin} onEdit={onEdit} onRemove={onRemove} />
      }
    >
      <li className="flex items-center gap-3 rounded-xl border bg-card p-2.5">
        <span className="flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted text-muted-foreground">
          {icon ? (
            <img src={icon} alt="" className="size-full object-cover [image-rendering:pixelated]" />
          ) : (
            <ServerIcon className="size-5" />
          )}
        </span>
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <p className="truncate text-sm font-semibold">{server.name}</p>
            <p className="truncate font-mono text-xs text-muted-foreground">{server.address}</p>
          </div>
          <p className="truncate text-xs text-muted-foreground">
            {status.isPending
              ? t('servers.pinging')
              : status.error
                ? status.error.message
                : status.data.motd.replace(/\s+/g, ' ').trim()}
          </p>
        </div>
        {status.data && (
          <div className="shrink-0 text-right text-xs text-muted-foreground">
            <p>
              {t('servers.players', { online: status.data.playersOnline, count: status.data.playersMax })}
            </p>
            <p>
              {t('servers.versionLatency', { version: status.data.version, latency: status.data.latencyMs })}
            </p>
          </div>
        )}
        <div className="flex items-center gap-1">
          <Button
            size="icon"
            variant="ghost"
            className="text-primary"
            title={t('servers.join')}
            aria-label={t('servers.join')}
            disabled={!canJoin}
            onClick={onJoin}
          >
            <Play />
          </Button>
          <Button
            size="icon"
            variant="ghost"
            className="text-destructive hover:text-destructive"
            title={t('common:action.remove')}
            aria-label={t('item.remove', { name: server.name })}
            onClick={onRemove}
          >
            <Trash2 />
          </Button>
        </div>
      </li>
    </ContextMenuArea>
  )
}
