import { Archive, Copy, FolderOpen, RefreshCw, Trash2 } from 'lucide-react'
import { useState } from 'react'
import type { World } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button } from '@/components/ui/button'
import { openSavesFolder, useBackupWorld, useDeleteWorld, useWorlds } from '@/hooks/use-worlds'

export function WorldsPage({ instanceId }: { instanceId: string }) {
  const { data: worlds = [], error, isPending, refetch } = useWorlds(instanceId)
  const remove = useDeleteWorld(instanceId)
  const backup = useBackupWorld(instanceId)
  const [pendingRemoval, setPendingRemoval] = useState<World | null>(null)

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">{worlds.length} worlds</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> Refresh
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openSavesFolder(instanceId)}>
            <FolderOpen /> Open Folder
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}
      {backup.data && <p className="text-sm text-muted-foreground">Backup saved to {backup.data}</p>}
      {backup.error && <p className="text-sm text-destructive">{backup.error.message}</p>}

      <div className="min-h-0 flex-1 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background text-left text-muted-foreground">
            <tr className="border-b">
              <th className="p-2 font-medium">Name</th>
              <th className="p-2 font-medium">Mode</th>
              <th className="p-2 font-medium">Version</th>
              <th className="p-2 font-medium">Last played</th>
              <th className="p-2 font-medium">Seed</th>
              <th className="w-20 p-2" />
            </tr>
          </thead>
          <tbody>
            {worlds.map((world) => (
              <tr key={world.folder} className="border-b last:border-0 hover:bg-accent/50">
                <td className="p-2">
                  <p className="font-medium">{world.name}</p>
                  {world.name !== world.folder && (
                    <p className="text-xs text-muted-foreground">{world.folder}</p>
                  )}
                </td>
                <td className="p-2 capitalize">{world.hardcore ? 'Hardcore' : world.gameMode}</td>
                <td className="p-2">{world.gameVersion}</td>
                <td className="p-2">
                  {world.lastPlayed ? new Date(world.lastPlayed).toLocaleString() : '—'}
                </td>
                <td className="p-2">
                  {world.seed && (
                    <button
                      type="button"
                      title="Copy seed"
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
                      aria-label={`Back up ${world.name}`}
                      title="Back up"
                      className="hover:text-foreground disabled:opacity-50"
                      disabled={backup.isPending}
                      onClick={() => backup.mutate(world.folder)}
                    >
                      <Archive className="size-4" />
                    </button>
                    <button
                      type="button"
                      aria-label={`Delete ${world.name}`}
                      title="Delete"
                      className="hover:text-destructive"
                      onClick={() => setPendingRemoval(world)}
                    >
                      <Trash2 className="size-4" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {isPending && <p className="p-3 text-sm text-muted-foreground">Reading worlds…</p>}
        {!isPending && worlds.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">No worlds yet.</p>
        )}
      </div>

      {pendingRemoval && (
        <ConfirmDialog
          title={`Delete ${pendingRemoval.name}?`}
          description="The world will be permanently removed. Consider making a backup first."
          confirmLabel="Delete"
          onConfirm={() => remove.mutate(pendingRemoval.folder, { onSuccess: () => setPendingRemoval(null) })}
          onClose={() => setPendingRemoval(null)}
        />
      )}
    </div>
  )
}
