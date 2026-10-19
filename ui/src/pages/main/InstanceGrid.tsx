import { ChevronDown, ChevronRight } from 'lucide-react'
import { useMemo, useState, type DragEvent, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useSession } from '@/app/sessions'
import type { InstanceSummary } from '@/bindings'
import { InstanceIcon } from '@/components/InstanceIcon'
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu'
import { useSetInstanceGroup } from '@/hooks/use-instance-actions'
import { useStoredState } from '@/hooks/use-stored-state'
import { cn } from '@/lib/utils'
import { InstanceTileMenu } from '@/pages/main/InstanceTileMenu'

export type InstanceSort = 'name' | 'lastPlayed'

const DRAG_TYPE = 'application/x-jelauncher-instance'
const UNGROUPED = ''

interface InstanceGridProps {
  instances: InstanceSummary[]
  search: string
  sort: InstanceSort
  selectedId: string | null
  onSelect: (id: string) => void
}

export function InstanceGrid({ instances, search, sort, selectedId, onSelect }: InstanceGridProps) {
  const { t } = useTranslation('main')
  const [collapsed, setCollapsed] = useStoredState<string[]>('collapsed-groups', [])
  const groups = useMemo(() => groupInstances(instances, search, sort), [instances, search, sort])

  const toggle = (group: string) =>
    setCollapsed((current) =>
      current.includes(group) ? current.filter((g) => g !== group) : [...current, group],
    )

  if (groups.length === 0) return <p className="text-muted-foreground">{t('grid.noMatch', { search })}</p>

  return (
    <div className="flex flex-col gap-4">
      {groups.map(([group, members]) => (
        <InstanceGroup
          key={group}
          group={group}
          showHeader={groups.length > 1 || group !== UNGROUPED}
          isCollapsed={collapsed.includes(group)}
          onToggle={() => toggle(group)}
        >
          {members.map((instance) => (
            <InstanceTile
              key={instance.id}
              instance={instance}
              isSelected={instance.id === selectedId}
              onSelect={() => onSelect(instance.id)}
            />
          ))}
        </InstanceGroup>
      ))}
    </div>
  )
}

function groupInstances(instances: InstanceSummary[], search: string, sort: InstanceSort) {
  const query = search.trim().toLowerCase()
  const byGroup = new Map<string, InstanceSummary[]>()
  for (const instance of instances) {
    if (query && !instance.name.toLowerCase().includes(query)) continue
    const group = instance.group ?? UNGROUPED
    byGroup.set(group, [...(byGroup.get(group) ?? []), instance])
  }
  const compare =
    sort === 'name'
      ? (a: InstanceSummary, b: InstanceSummary) => a.name.localeCompare(b.name)
      : (a: InstanceSummary, b: InstanceSummary) => b.lastLaunchTime - a.lastLaunchTime
  return [...byGroup.entries()]
    .map(([group, members]) => [group, members.sort(compare)] as const)
    .sort(([a], [b]) => (a === UNGROUPED ? -1 : b === UNGROUPED ? 1 : a.localeCompare(b)))
}

interface InstanceGroupProps {
  group: string
  showHeader: boolean
  isCollapsed: boolean
  onToggle: () => void
  children: ReactNode
}

function InstanceGroup({ group, showHeader, isCollapsed, onToggle, children }: InstanceGroupProps) {
  const { t } = useTranslation('main')
  const setGroup = useSetInstanceGroup()
  const [isDropTarget, setIsDropTarget] = useState(false)

  const onDragOver = (event: DragEvent) => {
    if (!event.dataTransfer.types.includes(DRAG_TYPE)) return
    event.preventDefault()
    setIsDropTarget(true)
  }
  const onDrop = (event: DragEvent) => {
    setIsDropTarget(false)
    const id = event.dataTransfer.getData(DRAG_TYPE)
    if (id) setGroup.mutate({ id, group: group || null })
  }

  return (
    <section
      onDragOver={onDragOver}
      onDragLeave={() => setIsDropTarget(false)}
      onDrop={onDrop}
      className={cn('rounded-md', isDropTarget && 'bg-accent/40 ring-1 ring-ring')}
    >
      {showHeader && (
        <button
          type="button"
          onClick={onToggle}
          className="mb-2 flex w-full items-center gap-1 border-b pb-1 text-left text-sm font-medium text-muted-foreground"
        >
          {isCollapsed ? <ChevronRight className="size-4" /> : <ChevronDown className="size-4" />}
          {group || t('grid.ungrouped')}
        </button>
      )}
      {!isCollapsed && (
        <ul className="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-3">{children}</ul>
      )}
    </section>
  )
}

interface InstanceTileProps {
  instance: InstanceSummary
  isSelected: boolean
  onSelect: () => void
}

function InstanceTile({ instance, isSelected, onSelect }: InstanceTileProps) {
  const { session, launch } = useSession(instance.id)
  const isActive = session.status !== 'idle'

  return (
    <li>
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button
            type="button"
            draggable
            onDragStart={(event) => event.dataTransfer.setData(DRAG_TYPE, instance.id)}
            onClick={onSelect}
            onContextMenu={onSelect}
            onDoubleClick={() => !isActive && void launch()}
            className={cn(
              'relative flex w-full flex-col items-center gap-2 rounded-md p-3 text-center hover:bg-accent',
              isSelected && 'bg-accent ring-1 ring-ring',
            )}
          >
            <InstanceIcon iconKey={instance.iconKey} className="size-12" />
            <span className="line-clamp-2 text-sm">{instance.name}</span>
            {isActive && <span className="absolute top-2 right-2 size-2.5 rounded-full bg-green-500" />}
          </button>
        </ContextMenuTrigger>
        <InstanceTileMenu instance={instance} />
      </ContextMenu>
    </li>
  )
}
