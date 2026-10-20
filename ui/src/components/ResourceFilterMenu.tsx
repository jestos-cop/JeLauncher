import { ListFilter } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { NO_FILTERS, type ResourceFilters, SOURCES, STATUSES, TRAITS } from '@/lib/resource-filters'

interface ResourceFilterMenuProps {
  filters: ResourceFilters
  onChange: (filters: ResourceFilters) => void
  /** Offers the source filters; off when sources are not tracked. */
  withSources: boolean
  /** Offers "Update available"; only after updates were checked. */
  withUpdates: boolean
}

export function ResourceFilterMenu({ filters, onChange, withSources, withUpdates }: ResourceFilterMenuProps) {
  const { t } = useTranslation('instance')
  const count = filters.statuses.length + filters.sources.length + filters.traits.length
  const traits = TRAITS.filter(
    (trait) => (trait !== 'hasUpdate' || withUpdates) && (trait !== 'locked' || withSources),
  )
  const toggle = <T,>(values: T[], value: T, checked: boolean) =>
    checked ? [...values, value] : values.filter((other) => other !== value)

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="sm" variant={count > 0 ? 'secondary' : 'ghost'}>
          <ListFilter /> {t('resources.filter.button')}
          {count > 0 && <Badge className="h-4 min-w-4 px-1 text-[10px]">{count}</Badge>}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-max">
        <DropdownMenuLabel>{t('resources.filter.status')}</DropdownMenuLabel>
        {STATUSES.map((status) => (
          <DropdownMenuCheckboxItem
            key={status}
            checked={filters.statuses.includes(status)}
            onSelect={(event) => event.preventDefault()}
            onCheckedChange={(checked) =>
              onChange({ ...filters, statuses: toggle(filters.statuses, status, checked) })
            }
          >
            {t(`resources.filter.${status}`)}
          </DropdownMenuCheckboxItem>
        ))}
        {withSources && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuLabel>{t('resources.filter.source')}</DropdownMenuLabel>
            {SOURCES.map((source) => (
              <DropdownMenuCheckboxItem
                key={source}
                checked={filters.sources.includes(source)}
                onSelect={(event) => event.preventDefault()}
                onCheckedChange={(checked) =>
                  onChange({ ...filters, sources: toggle(filters.sources, source, checked) })
                }
              >
                {t(`resources.filter.${source}`)}
              </DropdownMenuCheckboxItem>
            ))}
          </>
        )}
        <DropdownMenuSeparator />
        <DropdownMenuLabel>{t('resources.filter.only')}</DropdownMenuLabel>
        {traits.map((trait) => (
          <DropdownMenuCheckboxItem
            key={trait}
            checked={filters.traits.includes(trait)}
            onSelect={(event) => event.preventDefault()}
            onCheckedChange={(checked) =>
              onChange({ ...filters, traits: toggle(filters.traits, trait, checked) })
            }
          >
            {t(`resources.filter.${trait}`)}
          </DropdownMenuCheckboxItem>
        ))}
        {count > 0 && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={() => onChange(NO_FILTERS)}>
              {t('resources.filter.clear')}
            </DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
