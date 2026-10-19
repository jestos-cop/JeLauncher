import { convertFileSrc } from '@tauri-apps/api/core'
import { FolderOpen, Pencil, RefreshCw, Trash2 } from 'lucide-react'
import { useState } from 'react'
import type { Screenshot } from '@/bindings'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PromptDialog } from '@/components/PromptDialog'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import {
  openScreenshotsFolder,
  useDeleteScreenshot,
  useRenameScreenshot,
  useScreenshots,
} from '@/hooks/use-screenshots'

type PendingAction = { kind: 'view' | 'rename' | 'delete'; screenshot: Screenshot } | null

export function ScreenshotsPage({ instanceId }: { instanceId: string }) {
  const { data: screenshots = [], error, isPending, refetch } = useScreenshots(instanceId)
  const remove = useDeleteScreenshot(instanceId)
  const rename = useRenameScreenshot(instanceId)
  const [action, setAction] = useState<PendingAction>(null)
  const close = () => setAction(null)

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">{screenshots.length} screenshots</span>
        <div className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={() => refetch()}>
            <RefreshCw /> Refresh
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openScreenshotsFolder(instanceId)}>
            <FolderOpen /> Open Folder
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error.message}</p>}

      <div className="min-h-0 flex-1 overflow-auto">
        <div className="grid grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-3">
          {screenshots.map((screenshot) => (
            <figure key={screenshot.fileName} className="group relative overflow-hidden rounded-md border">
              <button
                type="button"
                className="block aspect-video w-full bg-muted"
                onClick={() => setAction({ kind: 'view', screenshot })}
              >
                <img
                  src={convertFileSrc(screenshot.path)}
                  alt={screenshot.fileName}
                  loading="lazy"
                  className="size-full object-cover"
                />
              </button>
              <figcaption className="flex items-center gap-2 px-2 py-1 text-xs">
                <span className="truncate" title={screenshot.fileName}>
                  {screenshot.fileName}
                </span>
                <div className="ml-auto flex shrink-0 gap-2 text-muted-foreground">
                  <button
                    type="button"
                    title="Rename"
                    aria-label={`Rename ${screenshot.fileName}`}
                    className="hover:text-foreground"
                    onClick={() => setAction({ kind: 'rename', screenshot })}
                  >
                    <Pencil className="size-3.5" />
                  </button>
                  <button
                    type="button"
                    title="Delete"
                    aria-label={`Delete ${screenshot.fileName}`}
                    className="hover:text-destructive"
                    onClick={() => setAction({ kind: 'delete', screenshot })}
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </div>
              </figcaption>
            </figure>
          ))}
        </div>
        {isPending && <p className="p-3 text-sm text-muted-foreground">Reading screenshots…</p>}
        {!isPending && screenshots.length === 0 && (
          <p className="p-3 text-sm text-muted-foreground">
            No screenshots yet. Press F2 in game to take one.
          </p>
        )}
      </div>

      {action?.kind === 'view' && (
        <Dialog open onOpenChange={(open) => !open && close()}>
          <DialogContent className="sm:max-w-5xl">
            <DialogHeader>
              <DialogTitle>{action.screenshot.fileName}</DialogTitle>
            </DialogHeader>
            <img
              src={convertFileSrc(action.screenshot.path)}
              alt={action.screenshot.fileName}
              className="max-h-[75vh] w-full rounded-md object-contain"
            />
          </DialogContent>
        </Dialog>
      )}
      {action?.kind === 'rename' && (
        <PromptDialog
          title="Rename Screenshot"
          initialValue={action.screenshot.fileName}
          confirmLabel="Rename"
          error={rename.error?.message}
          onConfirm={(newName) =>
            rename.mutate({ fileName: action.screenshot.fileName, newName }, { onSuccess: close })
          }
          onClose={close}
        />
      )}
      {action?.kind === 'delete' && (
        <ConfirmDialog
          title={`Delete ${action.screenshot.fileName}?`}
          description="The screenshot will be permanently removed."
          confirmLabel="Delete"
          onConfirm={() => remove.mutate(action.screenshot.fileName, { onSuccess: close })}
          onClose={close}
        />
      )}
    </div>
  )
}
