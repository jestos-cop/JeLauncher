import { save } from '@tauri-apps/plugin-dialog'
import { File, Folder } from 'lucide-react'
import { useState } from 'react'
import type { ExportEntry, InstanceSummary } from '@/bindings'
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
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { type ExportFormat, useExportEntries, useExportInstance } from '@/hooks/use-export'

interface ExportInstanceDialogProps {
  instance: InstanceSummary
  onClose: () => void
}

export function ExportInstanceDialog({ instance, onClose }: ExportInstanceDialogProps) {
  const [format, setFormat] = useState<ExportFormat>('zip')
  const [version, setVersion] = useState('1.0.0')
  const [summary, setSummary] = useState('')
  const [toggled, setToggled] = useState<Set<string>>(new Set())
  const entries = useExportEntries(instance.id)
  const exportInstance = useExportInstance(instance.id)

  const isBlocked = (entry: ExportEntry) => entry.blocked !== toggled.has(entry.path)
  const toggle = (path: string) =>
    setToggled((current) => {
      const next = new Set(current)
      if (!next.delete(path)) next.add(path)
      return next
    })

  const submit = async () => {
    const output = await save({
      defaultPath: `${instance.name}.${format}`,
      filters: [{ name: format === 'zip' ? 'Instance archive' : 'Modrinth pack', extensions: [format] }],
    })
    if (!output) return
    const blockedPaths = (entries.data ?? []).filter(isBlocked).map((entry) => entry.path)
    exportInstance.mutate({ format, output, blockedPaths, version, summary }, { onSuccess: onClose })
  }

  return (
    <Dialog open onOpenChange={(open) => !open && !exportInstance.isPending && onClose()}>
      <DialogContent className="flex max-h-[85vh] flex-col sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Export {instance.name}</DialogTitle>
          <DialogDescription>
            {format === 'zip'
              ? 'A full copy of the instance that can be imported again.'
              : 'Files published on Modrinth are linked, everything else is bundled as overrides.'}
          </DialogDescription>
        </DialogHeader>

        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="self-start"
          value={format}
          onValueChange={(next) => next && setFormat(next as ExportFormat)}
        >
          <ToggleGroupItem value="zip">Instance (.zip)</ToggleGroupItem>
          <ToggleGroupItem value="mrpack">Modrinth (.mrpack)</ToggleGroupItem>
        </ToggleGroup>

        {format === 'mrpack' && (
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
            <Label htmlFor="export-version">Version</Label>
            <Input id="export-version" value={version} onChange={(event) => setVersion(event.target.value)} />
            <Label htmlFor="export-summary">Summary</Label>
            <Input
              id="export-summary"
              placeholder="Optional"
              value={summary}
              onChange={(event) => setSummary(event.target.value)}
            />
          </div>
        )}

        <Label>Included files</Label>
        <ScrollArea className="min-h-0 flex-1 rounded-md border">
          <div className="flex flex-col p-1">
            {entries.data?.map((entry) => (
              <label
                key={entry.path}
                className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm hover:bg-accent"
              >
                <Checkbox checked={!isBlocked(entry)} onCheckedChange={() => toggle(entry.path)} />
                {entry.isDir ? (
                  <Folder className="size-4 text-muted-foreground" />
                ) : (
                  <File className="size-4 text-muted-foreground" />
                )}
                {entry.path}
              </label>
            ))}
            {entries.data?.length === 0 && (
              <p className="p-2 text-sm text-muted-foreground">The instance has no game files yet.</p>
            )}
          </div>
        </ScrollArea>

        {(entries.error ?? exportInstance.error) && (
          <p className="text-sm text-destructive">{(entries.error ?? exportInstance.error)?.message}</p>
        )}
        <DialogFooter>
          <Button variant="ghost" disabled={exportInstance.isPending} onClick={onClose}>
            Cancel
          </Button>
          <Button
            disabled={!entries.data || exportInstance.isPending || (format === 'mrpack' && !version.trim())}
            onClick={() => void submit()}
          >
            {exportInstance.isPending ? 'Exporting…' : 'Export…'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
