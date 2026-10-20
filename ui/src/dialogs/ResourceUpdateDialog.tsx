import { ChevronDown, ChevronRight, ExternalLink } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands, type ResourceFile, type ResourceUpdate } from '@/bindings'
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
import { unwrap } from '@/lib/command'
import { PROVIDER_NAMES } from '@/lib/providers'

interface ResourceUpdateDialogProps {
  updates: ResourceUpdate[]
  files: ResourceFile[]
  onApply: (updates: ResourceUpdate[]) => void
  onClose: () => void
}

/** Whether the launcher may download the update, or the author only allows their website. */
function isDownloadable(update: ResourceUpdate) {
  const file = update.version.files.find((candidate) => candidate.primary) ?? update.version.files[0]
  return Boolean(file?.url)
}

/** Lets the user review available updates, read their changelogs and pick which ones to apply. */
export function ResourceUpdateDialog({ updates, files, onApply, onClose }: ResourceUpdateDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const [selected, setSelected] = useState(
    () => new Set(updates.filter(isDownloadable).map((update) => update.fileName)),
  )
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
          <DialogTitle>{t('resourceUpdate.title')}</DialogTitle>
          <DialogDescription>{t('resourceUpdate.description')}</DialogDescription>
        </DialogHeader>
        <ul className="max-h-[60vh] overflow-auto rounded-md border">
          {updates.map((update) => {
            const file = fileFor(update)
            const isExpanded = expanded === update.fileName
            return (
              <li key={update.fileName} className="border-b last:border-0">
                <div className="flex items-center gap-3 p-2">
                  <Checkbox
                    disabled={!isDownloadable(update)}
                    checked={selected.has(update.fileName)}
                    aria-label={t('resourceUpdate.select', { name: file?.metadata?.name ?? update.fileName })}
                    onCheckedChange={(checked) => toggle(update.fileName, checked === true)}
                  />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{file?.metadata?.name ?? update.fileName}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {file?.metadata?.version ?? update.fileName} → {update.version.versionNumber} ·{' '}
                      {PROVIDER_NAMES[update.provider]}
                    </p>
                  </div>
                  {!isDownloadable(update) && file?.source && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => void unwrap(commands.openUrl(file.source?.homepage ?? ''))}
                    >
                      <ExternalLink /> {t('resourceUpdate.websiteOnly')}
                    </Button>
                  )}
                  {update.version.releaseType !== 'release' && (
                    <Badge variant="secondary">{t(`releaseType.${update.version.releaseType}`)}</Badge>
                  )}
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => setExpanded(isExpanded ? null : update.fileName)}
                  >
                    {isExpanded ? <ChevronDown /> : <ChevronRight />} {t('resourceUpdate.changelog')}
                  </Button>
                </div>
                {isExpanded && (
                  <p className="max-h-48 overflow-auto px-9 pb-3 text-xs whitespace-pre-wrap text-muted-foreground">
                    {update.version.changelog ?? t('resourceUpdate.noChangelog')}
                  </p>
                )}
              </li>
            )
          })}
        </ul>
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            {t('common:action.cancel')}
          </Button>
          <Button
            disabled={selected.size === 0}
            onClick={() => onApply(updates.filter((update) => selected.has(update.fileName)))}
          >
            {t('resourceUpdate.apply', { selected: selected.size, total: updates.length })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
