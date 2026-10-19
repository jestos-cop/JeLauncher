import { ArrowUp, Pencil, Play, Plus, RefreshCw, Server as ServerIcon, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { useSession } from '@/app/sessions'
import type { Server } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { ServerDialog } from '@/dialogs/ServerDialog'
import {
  useMoveServerUp,
  useRemoveServer,
  useSaveServer,
  useServerStatus,
  useServers,
} from '@/hooks/use-servers'

type PendingAction = { kind: 'edit'; index: number | null } | { kind: 'delete'; index: number } | null

export function ServersPage({ instanceId }: { instanceId: string }) {
  const { t } = useTranslation(['instance', 'common'])
  const queryClient = useQueryClient()
  const { session, launch } = useSession(instanceId)
  const { data: servers = [], error, isPending } = useServers(instanceId)
  const save = useSaveServer(instanceId)
  const remove = useRemoveServer(instanceId)
  const moveUp = useMoveServerUp(instanceId)
  const [action, setAction] = useState<PendingAction>(null)
  const close = () => {
    save.reset()
    setAction(null)
  }
  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ['servers', instanceId] })
    void queryClient.invalidateQueries({ queryKey: ['server-status'] })
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm text-muted-foreground">{t('servers.count', { count: servers.length })}</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={refresh}>
            <RefreshCw /> {t('common:action.refresh')}
          </Button>
          <Button size="sm" onClick={() => setAction({ kind: 'edit', index: null })}>
            <Plus /> {t('servers.add')}
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        {servers.map((server, index) => (
          <ServerRow
            key={`${index}-${server.address}`}
            server={server}
            canMoveUp={index > 0}
            canJoin={session.status === 'idle'}
            onJoin={() => void launch({ quickPlay: { type: 'server', target: server.address }, demo: false })}
            onEdit={() => setAction({ kind: 'edit', index })}
            onMoveUp={() => moveUp.mutate(index)}
            onDelete={() => setAction({ kind: 'delete', index })}
          />
        ))}
        {isPending && <p className="p-3 text-sm text-muted-foreground">{t('servers.reading')}</p>}
        {!isPending && servers.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">{t('servers.empty')}</p>
        )}
      </div>

      {action?.kind === 'edit' && (
        <ServerDialog
          server={action.index === null ? null : (servers[action.index] ?? null)}
          error={save.error?.message}
          pending={save.isPending}
          onSave={(server) => save.mutate({ index: action.index, server }, { onSuccess: close })}
          onClose={close}
        />
      )}
      {action?.kind === 'delete' && (
        <ConfirmDialog
          title={t('confirm.remove', { name: servers[action.index]?.name ?? t('servers.unnamed') })}
          description={t('servers.removeDescription')}
          confirmLabel={t('common:action.remove')}
          onConfirm={() => remove.mutate(action.index, { onSuccess: close })}
          onClose={close}
        />
      )}
    </div>
  )
}

interface ServerRowProps {
  server: Server
  canMoveUp: boolean
  canJoin: boolean
  onJoin: () => void
  onEdit: () => void
  onMoveUp: () => void
  onDelete: () => void
}

function ServerRow({ server, canMoveUp, canJoin, onJoin, onEdit, onMoveUp, onDelete }: ServerRowProps) {
  const { t } = useTranslation(['instance', 'common'])
  const status = useServerStatus(server.address)
  const icon = status.data?.favicon ?? (server.icon && `data:image/png;base64,${server.icon}`)

  return (
    <div className="flex items-center gap-3 border-b p-2 last:border-0 hover:bg-accent/50">
      {icon ? (
        <img src={icon} alt="" className="size-12 shrink-0 rounded [image-rendering:pixelated]" />
      ) : (
        <ServerIcon className="size-12 shrink-0 p-2 text-muted-foreground" />
      )}
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <p className="truncate font-medium">{server.name}</p>
          <p className="truncate text-xs text-muted-foreground">{server.address}</p>
        </div>
        <p className="truncate whitespace-pre text-xs text-muted-foreground">
          {status.isPending
            ? t('servers.pinging')
            : status.error
              ? status.error.message
              : status.data.motd.replace(/\s+/g, ' ').trim()}
        </p>
      </div>
      {status.data && (
        <div className="shrink-0 text-right text-xs text-muted-foreground">
          <p>{t('servers.players', { online: status.data.playersOnline, count: status.data.playersMax })}</p>
          <p>
            {t('servers.versionLatency', { version: status.data.version, latency: status.data.latencyMs })}
          </p>
        </div>
      )}
      <Button size="sm" variant="outline" disabled={!canJoin} onClick={onJoin}>
        <Play /> {t('servers.join')}
      </Button>
      <div className="flex shrink-0 gap-2 text-muted-foreground">
        <button
          type="button"
          title={t('servers.moveUp')}
          aria-label={t('item.moveUp', { name: server.name })}
          className="hover:text-foreground disabled:opacity-30"
          disabled={!canMoveUp}
          onClick={onMoveUp}
        >
          <ArrowUp className="size-4" />
        </button>
        <button
          type="button"
          title={t('common:action.edit')}
          aria-label={t('item.edit', { name: server.name })}
          className="hover:text-foreground"
          onClick={onEdit}
        >
          <Pencil className="size-4" />
        </button>
        <button
          type="button"
          title={t('common:action.remove')}
          aria-label={t('item.remove', { name: server.name })}
          className="hover:text-destructive"
          onClick={onDelete}
        >
          <Trash2 className="size-4" />
        </button>
      </div>
    </div>
  )
}
