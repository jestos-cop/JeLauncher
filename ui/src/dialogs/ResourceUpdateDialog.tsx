import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import type { ResourceFile, ResourceUpdate } from '@/bindings'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface ResourceUpdateDialogProps {
  updates: ResourceUpdate[]
  files: ResourceFile[]
  onApply: (updates: ResourceUpdate[]) => void
  onClose: () => void
}

/** Lets the user review available updates, read their changelogs and pick which ones to apply. */
export function ResourceUpdateDialog({ updates, files, onApply, onClose }: ResourceUpdateDialogProps) {
  const [selected, setSelected] = useState(() => new Set(updates.map((update) => update.fileName)))
  const [expanded, setExpanded] = useState<string | null>(null)
  const fileFor = (update: ResourceUpdate) => files.find((file) => file.fileName === update.fileName)

  const toggle = (fileName: string, checked: boolean) =>
    setSelected((current) => {
      const next = new Set(current)
      if (checked) next.add(fileName)
      else next.delete(fileName)
      return next
    })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Review Updates</DialogTitle>
          <DialogDescription>Pick the updates to install. Disabled files stay disabled.</DialogDescription>
        </DialogHeader>
        <ul className="max-h-[60vh] overflow-auto rounded-md border">
          {updates.map((update) => {
            const file = fileFor(update)
            const isExpanded = expanded === update.fileName
            return (
              <li key={update.fileName} className="border-b last:border-0">
                <div className="flex items-center gap-3 p-2">
                  <Checkbox
                    checked={selected.has(update.fileName)}
                    aria-label={`Update ${file?.metadata?.name ?? update.fileName}`}
                    onCheckedChange={(checked) => toggle(update.fileName, checked === true)}
                  />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{file?.metadata?.name ?? update.fileName}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {file?.metadata?.version ?? update.fileName} → {update.version.versionNumber}
                    </p>
                  </div>
                  {update.version.releaseType !== 'release' && (
                    <Badge variant="secondary">{update.version.releaseType}</Badge>
                  )}
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => setExpanded(isExpanded ? null : update.fileName)}
                  >
                    {isExpanded ? <ChevronDown /> : <ChevronRight />} Changelog
                  </Button>
                </div>
                {isExpanded && (
                  <p className="max-h-48 overflow-auto px-9 pb-3 text-xs whitespace-pre-wrap text-muted-foreground">
                    {update.version.changelog ?? 'No changelog for this version.'}
                  </p>
                )}
              </li>
            )
          })}
        </ul>
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            disabled={selected.size === 0}
            onClick={() => onApply(updates.filter((update) => selected.has(update.fileName)))}
          >
            Update {selected.size} of {updates.length}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
