import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { ResourceFile, ResourceKind } from '@/bindings'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { useRemovalPlan, useRemoveResources } from '@/hooks/use-resources'

interface RemoveResourceDialogProps {
  instanceId: string
  kind: ResourceKind
  file: ResourceFile
  files: ResourceFile[]
  nameOf: (fileName: string) => string
  onClose: () => void
}

/** Confirms removing a file and offers to take along the mods that need it and the
 * dependencies nothing else needs. */
export function RemoveResourceDialog({
  instanceId,
  kind,
  file,
  files,
  nameOf,
  onClose,
}: RemoveResourceDialogProps) {
  const { t } = useTranslation(['instance', 'common'])
  const remove = useRemoveResources(instanceId, kind)
  const [keptDependents, setKeptDependents] = useState<Set<string>>(new Set())
  const [keptUnused, setKeptUnused] = useState<Set<string>>(new Set())
  const { data: direct } = useRemovalPlan(instanceId, kind, [file.fileName])
  const dependents = direct?.dependents ?? []
  const removedDependents = dependents.filter((name) => !keptDependents.has(name))
  const { data: plan } = useRemovalPlan(instanceId, kind, [file.fileName, ...removedDependents])
  const unused = plan?.unusedDependencies ?? []
  const removedUnused = unused.filter((name) => !keptUnused.has(name))
  const toRemove = [file.fileName, ...removedDependents, ...removedUnused]
  const toDisable = dependents.filter(
    (name) => keptDependents.has(name) && files.some((other) => other.fileName === name && other.enabled),
  )

  return (
    <AlertDialog open onOpenChange={(open) => !open && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('confirm.remove', { name: nameOf(file.fileName) })}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('resources.removeDescription', { file: file.fileName })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        {dependents.length > 0 && (
          <FileChoices
            title={t('resources.removal.dependents')}
            hint={t('resources.removal.dependentsHint')}
            fileNames={dependents}
            kept={keptDependents}
            onKeptChange={setKeptDependents}
            nameOf={nameOf}
          />
        )}
        {unused.length > 0 && (
          <FileChoices
            title={t('resources.removal.unused')}
            hint={t('resources.removal.unusedHint')}
            fileNames={unused}
            kept={keptUnused}
            onKeptChange={setKeptUnused}
            nameOf={nameOf}
          />
        )}
        {remove.error && <p className="text-sm text-destructive">{remove.error.message}</p>}
        <AlertDialogFooter>
          <AlertDialogCancel>{t('common:action.cancel')}</AlertDialogCancel>
          <Button
            variant="destructive"
            disabled={remove.isPending}
            onClick={() => remove.mutate({ remove: toRemove, disable: toDisable }, { onSuccess: onClose })}
          >
            {toRemove.length > 1
              ? t('resources.removal.confirm', { total: toRemove.length })
              : t('common:action.remove')}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

interface FileChoicesProps {
  title: string
  hint: string
  fileNames: string[]
  kept: Set<string>
  onKeptChange: (kept: Set<string>) => void
  nameOf: (fileName: string) => string
}

/** Files offered for removal, all checked until the user keeps some. */
function FileChoices({ title, hint, fileNames, kept, onKeptChange, nameOf }: FileChoicesProps) {
  return (
    <div className="flex flex-col gap-1.5">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{hint}</p>
      <ul className="max-h-40 overflow-auto rounded-md border">
        {fileNames.map((fileName) => (
          <li key={fileName} className="flex items-center gap-2 border-b px-2 py-1.5 text-sm last:border-0">
            <Checkbox
              checked={!kept.has(fileName)}
              aria-label={nameOf(fileName)}
              onCheckedChange={(checked) => {
                const next = new Set(kept)
                if (checked === true) next.delete(fileName)
                else next.add(fileName)
                onKeptChange(next)
              }}
            />
            <span className="min-w-0 flex-1 truncate">{nameOf(fileName)}</span>
            <span className="truncate text-xs text-muted-foreground">{fileName}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
