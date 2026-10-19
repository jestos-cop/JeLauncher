import { Package } from 'lucide-react'
import type { ProjectHit } from '@/bindings'
import { cn } from '@/lib/utils'

interface ProjectRowProps {
  hit: ProjectHit
  isSelected: boolean
  onSelect: () => void
}

export function ProjectRow({ hit, isSelected, onSelect }: ProjectRowProps) {
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
            <span className="text-xs text-muted-foreground">by {hit.author}</span>
            <span className="ml-auto text-xs text-muted-foreground">
              {Intl.NumberFormat(undefined, { notation: 'compact' }).format(hit.downloads)} downloads
            </span>
          </div>
          <p className="line-clamp-2 text-sm text-muted-foreground">{hit.description}</p>
        </div>
      </button>
    </li>
  )
}
