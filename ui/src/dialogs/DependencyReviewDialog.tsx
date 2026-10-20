import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { PlannedDependency, ProjectVersion } from '@/bindings'
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

interface DependencyReviewDialogProps {
  /** Name and project id of the version being installed. */
  name: string
  projectId: string
  dependencies: PlannedDependency[]
  onInstall: (dependencies: ProjectVersion[]) => void
  onClose: () => void
}

/** Lists what a version brings along so the user can leave dependencies out. */
export function DependencyReviewDialog({
  name,
  projectId,
  dependencies,
  onInstall,
  onClose,
}: DependencyReviewDialogProps) {
  const { t } = useTranslation(['instanceDialogs', 'common'])
  const [checked, setChecked] = useState(
    () => new Set(dependencies.filter((dep) => dep.kind === 'required').map((dep) => dep.projectId)),
  )
  const included = includedDependencies(projectId, dependencies, checked)
  const toInstall = dependencies.filter((dep) => included.has(dep.projectId) && dep.version && !dep.installed)
  const nameOf = (id: string) =>
    id === projectId ? name : (dependencies.find((dep) => dep.projectId === id)?.name ?? id)

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('dependencies.title', { name })}</DialogTitle>
          <DialogDescription>{t('dependencies.description')}</DialogDescription>
        </DialogHeader>
        <ul className="max-h-80 overflow-auto rounded-md border">
          {dependencies.map((dep) => {
            const reachable = dep.requiredBy.some((parent) => parent === projectId || included.has(parent))
            const selectable = reachable && dep.version !== null && !dep.installed
            return (
              <li key={dep.projectId} className="flex items-start gap-2 border-b p-2 last:border-0">
                <Checkbox
                  className="mt-0.5"
                  checked={dep.installed || (selectable && checked.has(dep.projectId))}
                  disabled={!selectable}
                  aria-label={dep.name}
                  onCheckedChange={(value) =>
                    setChecked((current) => {
                      const next = new Set(current)
                      if (value === true) next.add(dep.projectId)
                      else next.delete(dep.projectId)
                      return next
                    })
                  }
                />
                <div className={selectable || dep.installed ? 'min-w-0 flex-1' : 'min-w-0 flex-1 opacity-50'}>
                  <p className="flex items-center gap-2 text-sm font-medium">
                    {dep.name}
                    {dep.kind === 'optional' && (
                      <Badge variant="secondary">{t('dependencies.optional')}</Badge>
                    )}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {dep.installed
                      ? t('dependencies.installed')
                      : dep.version
                        ? dep.version.versionNumber
                        : t('dependencies.noVersion')}
                    {' · '}
                    {t('dependencies.neededBy', { names: dep.requiredBy.map(nameOf).join(', ') })}
                  </p>
                </div>
              </li>
            )
          })}
        </ul>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t('common:action.cancel')}
          </Button>
          <Button onClick={() => onInstall(toInstall.flatMap((dep) => (dep.version ? [dep.version] : [])))}>
            {t('dependencies.install', { total: toInstall.length + 1 })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/** Checked dependencies still linked to the installed version through other included ones. */
function includedDependencies(root: string, dependencies: PlannedDependency[], checked: Set<string>) {
  const included = new Set<string>()
  let changed = true
  while (changed) {
    changed = false
    for (const dep of dependencies) {
      if (
        !included.has(dep.projectId) &&
        (checked.has(dep.projectId) || dep.installed) &&
        dep.requiredBy.some((parent) => parent === root || included.has(parent))
      ) {
        included.add(dep.projectId)
        changed = true
      }
    }
  }
  return included
}
