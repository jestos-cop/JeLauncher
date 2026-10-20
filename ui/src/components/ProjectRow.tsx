import { Package } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { ProjectHit } from '@/bindings'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface ProjectRowProps {
  hit: ProjectHit
  isSelected: boolean
  onSelect: () => void
  /** The instance already has this project. */
  installed?: boolean
}

export function ProjectRow({ hit, isSelected, onSelect, installed = false }: ProjectRowProps) {
  const { t } = useTranslation(['newInstance', 'instanceDialogs'])
  return (
    <li>
      <button
        type="button"
        onClick={onSelect}
        className={cn('flex w-full gap-3 p-3 text-left hover:bg-accent', isSelected && 'bg-accent')}
      >
        {hit.iconUrl ? (
          <img src={hit.iconUrl} alt="" className="size-10 shrink-0 rounded-md" />
        ) : (
          <Package className="size-10 shrink-0 text-muted-foreground" />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className="font-medium">{hit.title}</span>
            {installed && (
              <Badge variant="secondary">{t('instanceDialogs:resourceDownload.installedBadge')}</Badge>
            )}
            {hit.author && (
              <span className="text-xs text-muted-foreground">
                {t('projectRow.author', { author: hit.author })}
              </span>
            )}
            {hit.downloads > 0 && (
              <span className="ml-auto text-xs text-muted-foreground">
                {t('projectRow.downloads', {
                  count: hit.downloads,
                  downloads: Intl.NumberFormat(undefined, { notation: 'compact' }).format(hit.downloads),
                })}
              </span>
            )}
          </div>
          <p className="line-clamp-2 text-sm text-muted-foreground">{hit.description}</p>
        </div>
      </button>
    </li>
  )
}
