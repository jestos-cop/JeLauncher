import { useMemo, useState } from 'react'
import type { VersionEntry, VersionType } from '@/bindings'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useMinecraftVersions } from '@/hooks/use-minecraft-versions'
import { cn } from '@/lib/utils'

const TYPE_FILTERS: { type: VersionType; label: string }[] = [
  { type: 'release', label: 'Releases' },
  { type: 'snapshot', label: 'Snapshots' },
  { type: 'old_beta', label: 'Betas' },
  { type: 'old_alpha', label: 'Alphas' },
]

interface VersionListProps {
  selected: string | null
  onSelect: (versionId: string) => void
}

export function VersionList({ selected, onSelect }: VersionListProps) {
  const { data, error, isPending } = useMinecraftVersions()
  const [search, setSearch] = useState('')
  const [types, setTypes] = useState<Set<VersionType>>(new Set(['release']))

  const versions = useMemo(() => {
    const query = search.trim().toLowerCase()
    return (data?.versions ?? []).filter(
      (version) => types.has(version.type) && version.id.toLowerCase().includes(query),
    )
  }, [data, search, types])

  const toggleType = (type: VersionType, checked: boolean) => {
    setTypes((current) => {
      const next = new Set(current)
      if (checked) next.add(type)
      else next.delete(type)
      return next
    })
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-4">
        <Input
          className="w-48"
          placeholder="Search versions"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        {TYPE_FILTERS.map(({ type, label }) => (
          <Label key={type} className="gap-2 font-normal">
            <Checkbox
              checked={types.has(type)}
              onCheckedChange={(checked) => toggleType(type, checked === true)}
            />
            {label}
          </Label>
        ))}
      </div>

      <ScrollArea className="min-h-0 flex-1 rounded-md border">
        {isPending && <p className="p-3 text-sm text-muted-foreground">Loading versions…</p>}
        {error && <p className="p-3 text-sm text-destructive">{error.message}</p>}
        <ul>
          {versions.map((version) => (
            <VersionRow
              key={version.id}
              version={version}
              isLatest={version.id === data?.latest.release || version.id === data?.latest.snapshot}
              isSelected={version.id === selected}
              onSelect={onSelect}
            />
          ))}
        </ul>
      </ScrollArea>
    </div>
  )
}

interface VersionRowProps {
  version: VersionEntry
  isLatest: boolean
  isSelected: boolean
  onSelect: (versionId: string) => void
}

function VersionRow({ version, isLatest, isSelected, onSelect }: VersionRowProps) {
  return (
    <li>
      <button
        type="button"
        onClick={() => onSelect(version.id)}
        className={cn(
          'flex w-full items-center gap-3 px-3 py-1.5 text-left text-sm hover:bg-accent',
          isSelected && 'bg-accent',
        )}
      >
        <span className="flex-1 font-medium">{version.id}</span>
        {isLatest && <Badge>Latest</Badge>}
        <span className="w-20 text-muted-foreground">{version.type.replace('old_', '')}</span>
        <span className="w-24 text-right text-muted-foreground">
          {new Date(version.releaseTime).toLocaleDateString()}
        </span>
      </button>
    </li>
  )
}
