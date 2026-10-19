import { save } from '@tauri-apps/plugin-dialog'
import { File, Folder } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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

const EXTENSIONS: Record<ExportFormat, string> = {
  zip: 'zip',
  mrpack: 'mrpack',
  curseforge: 'zip',
}

export function ExportInstanceDialog({ instance, onClose }: ExportInstanceDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
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
    const extension = EXTENSIONS[format]
    const output = await save({
      defaultPath: `${instance.name}.${extension}`,
      filters: [{ name: t(`export.format.${format}.label`), extensions: [extension] }],
    })
    if (!output) return
    const blockedPaths = (entries.data ?? []).filter(isBlocked).map((entry) => entry.path)
    exportInstance.mutate({ format, output, blockedPaths, version, summary }, { onSuccess: onClose })
  }

  return (
    <Dialog open onOpenChange={(open) => !open && !exportInstance.isPending && onClose()}>
      <DialogContent className="flex max-h-[85vh] flex-col sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('export.title', { name: instance.name })}</DialogTitle>
          <DialogDescription>{t(`export.format.${format}.description`)}</DialogDescription>
        </DialogHeader>

        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          className="self-start"
          value={format}
          onValueChange={(next) => next && setFormat(next as ExportFormat)}
        >
          <ToggleGroupItem value="zip">{t('export.instanceZip')}</ToggleGroupItem>
          <ToggleGroupItem value="mrpack">Modrinth (.mrpack)</ToggleGroupItem>
          <ToggleGroupItem value="curseforge">CurseForge (.zip)</ToggleGroupItem>
        </ToggleGroup>

        {format !== 'zip' && (
          <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2">
            <Label htmlFor="export-version">{t('export.version')}</Label>
            <Input id="export-version" value={version} onChange={(event) => setVersion(event.target.value)} />
            <Label htmlFor="export-summary">
              {format === 'mrpack' ? t('export.summary') : t('export.author')}
            </Label>
            <Input
              id="export-summary"
              placeholder={t('export.optional')}
              value={summary}
              onChange={(event) => setSummary(event.target.value)}
            />
          </div>
        )}

        <Label>{t('export.includedFiles')}</Label>
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
              <p className="p-2 text-sm text-muted-foreground">{t('export.noFiles')}</p>
            )}
          </div>
        </ScrollArea>

        {(entries.error ?? exportInstance.error) && (
          <p className="text-sm text-destructive">{(entries.error ?? exportInstance.error)?.message}</p>
        )}
        <DialogFooter>
          <Button variant="ghost" disabled={exportInstance.isPending} onClick={onClose}>
            {t('common:action.cancel')}
          </Button>
          <Button
            disabled={!entries.data || exportInstance.isPending || (format !== 'zip' && !version.trim())}
            onClick={() => void submit()}
          >
            {exportInstance.isPending ? t('export.exporting') : t('export.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
