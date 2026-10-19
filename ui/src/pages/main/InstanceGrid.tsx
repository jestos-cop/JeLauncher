import { Box, Play, Square } from 'lucide-react'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { cn } from '@/lib/utils'

interface InstanceGridProps {
  instances: InstanceSummary[]
  selectedId: string | null
  onSelect: (id: string) => void
}

export function InstanceGrid({ instances, selectedId, onSelect }: InstanceGridProps) {
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-3">
      {instances.map((instance) => (
        <InstanceTile
          key={instance.id}
          instance={instance}
          isSelected={instance.id === selectedId}
          onSelect={() => onSelect(instance.id)}
        />
      ))}
    </ul>
  )
}

interface InstanceTileProps {
  instance: InstanceSummary
  isSelected: boolean
  onSelect: () => void
}

function InstanceTile({ instance, isSelected, onSelect }: InstanceTileProps) {
  const { session, launch, kill } = useSession(instance.id)
  const isActive = session.status !== 'idle'

  return (
    <li>
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button
            type="button"
            onClick={onSelect}
            onContextMenu={onSelect}
            onDoubleClick={() => !isActive && void launch()}
            className={cn(
              'relative flex w-full flex-col items-center gap-2 rounded-md p-3 text-center hover:bg-accent',
              isSelected && 'bg-accent ring-1 ring-ring',
            )}
          >
            <Box className="size-12 text-muted-foreground" />
            <span className="line-clamp-2 text-sm">{instance.name}</span>
            {isActive && (
              <span className="absolute top-2 right-2 size-2.5 rounded-full bg-green-500" />
            )}
          </button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          {isActive ? (
            <ContextMenuItem disabled={session.status !== 'running'} onSelect={() => void kill()}>
              <Square /> Kill
            </ContextMenuItem>
          ) : (
            <ContextMenuItem onSelect={() => void launch()}>
              <Play /> Launch
            </ContextMenuItem>
          )}
        </ContextMenuContent>
      </ContextMenu>
    </li>
  )
}
