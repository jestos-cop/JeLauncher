import { ArrowUp, Pencil, Plus, RefreshCw, Server as ServerIcon, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
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
  const queryClient = useQueryClient()
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
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">{servers.length} servers</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={refresh}>
            <RefreshCw /> Refresh
          </Button>
          <Button size="sm" onClick={() => setAction({ kind: 'edit', index: null })}>
            <Plus /> Add Server
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
            onEdit={() => setAction({ kind: 'edit', index })}
            onMoveUp={() => moveUp.mutate(index)}
            onDelete={() => setAction({ kind: 'delete', index })}
          />
        ))}
        {isPending && <p className="p-3 text-sm text-muted-foreground">Reading servers…</p>}
        {!isPending && servers.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">No servers yet.</p>
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
          title={`Remove ${servers[action.index]?.name ?? 'server'}?`}
          description="The server will be removed from this instance's multiplayer list."
          confirmLabel="Remove"
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
  onEdit: () => void
  onMoveUp: () => void
  onDelete: () => void
}

function ServerRow({ server, canMoveUp, onEdit, onMoveUp, onDelete }: ServerRowProps) {
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
            ? 'Pinging…'
            : status.error
              ? status.error.message
              : status.data.motd.replace(/\s+/g, ' ').trim()}
        </p>
      </div>
      {status.data && (
        <div className="shrink-0 text-right text-xs text-muted-foreground">
          <p>
            {status.data.playersOnline}/{status.data.playersMax} players
          </p>
          <p>
            {status.data.version} · {status.data.latencyMs} ms
          </p>
        </div>
      )}
      <div className="flex shrink-0 gap-2 text-muted-foreground">
        <button
          type="button"
          title="Move up"
          aria-label={`Move ${server.name} up`}
          className="hover:text-foreground disabled:opacity-30"
          disabled={!canMoveUp}
          onClick={onMoveUp}
        >
          <ArrowUp className="size-4" />
        </button>
        <button
          type="button"
          title="Edit"
          aria-label={`Edit ${server.name}`}
          className="hover:text-foreground"
          onClick={onEdit}
        >
          <Pencil className="size-4" />
        </button>
        <button
          type="button"
          title="Remove"
          aria-label={`Remove ${server.name}`}
          className="hover:text-destructive"
          onClick={onDelete}
        >
          <Trash2 className="size-4" />
        </button>
      </div>
    </div>
  )
}
