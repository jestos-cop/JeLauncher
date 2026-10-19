import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { VersionEntry, VersionType } from '@/bindings'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useMinecraftVersions } from '@/hooks/use-minecraft-versions'
import { cn } from '@/lib/utils'

const TYPE_FILTERS = ['release', 'snapshot', 'old_beta', 'old_alpha'] satisfies VersionType[]

type TypeKey<T extends VersionType> = T extends `old_${infer Rest}` ? Rest : T
const typeKey = <T extends VersionType>(type: T) => type.replace('old_', '') as TypeKey<T>

interface VersionListProps {
  selected: string | null
  onSelect: (versionId: string) => void
}

export function VersionList({ selected, onSelect }: VersionListProps) {
  const { t } = useTranslation('newInstance')
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
          placeholder={t('versionList.search')}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        {TYPE_FILTERS.map((type) => (
          <Label key={type} className="gap-2 font-normal">
            <Checkbox
              checked={types.has(type)}
              onCheckedChange={(checked) => toggleType(type, checked === true)}
            />
            {t(`versionList.filter.${typeKey(type)}`)}
          </Label>
        ))}
      </div>

      <ScrollArea className="min-h-0 flex-1 rounded-md border">
        {isPending && <p className="p-3 text-sm text-muted-foreground">{t('versionList.loadingVersions')}</p>}
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
  const { t } = useTranslation('newInstance')
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
        {isLatest && <Badge>{t('versionList.latest')}</Badge>}
        <span className="w-20 text-muted-foreground">{t(`versionList.type.${typeKey(version.type)}`)}</span>
        <span className="w-24 text-right text-muted-foreground">
          {new Date(version.releaseTime).toLocaleDateString()}
        </span>
      </button>
    </li>
  )
}
