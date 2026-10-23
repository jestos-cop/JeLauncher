import { ArrowLeft, RefreshCw } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { type OverviewTab, useNavigation } from '@/app/navigation'
import { ContextMenuArea } from '@/components/ContextMenuArea'
import { Button } from '@/components/ui/button'
import { ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { OverviewSort } from '@/pages/overview/sort'

interface OverviewLayoutProps {
  tab: OverviewTab
  summary: string
  sort: OverviewSort
  onSortChange: (sort: OverviewSort) => void
  onRefresh: () => void
  children: ReactNode
}

/** Page frame shared by the worlds and servers of all instances. */
export function OverviewLayout({
  tab,
  summary,
  sort,
  onSortChange,
  onRefresh,
  children,
}: OverviewLayoutProps) {
  const { t } = useTranslation(['main', 'common'])
  const { navigate } = useNavigation()
  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-3 border-b px-3 py-2">
        <Button size="sm" variant="ghost" onClick={() => navigate({ page: 'main' })}>
          <ArrowLeft /> {t('common:action.back')}
        </Button>
        <ToggleGroup
          type="single"
          variant="outline"
          size="sm"
          value={tab}
          onValueChange={(next) => next && navigate({ page: 'overview', tab: next as OverviewTab })}
        >
          <ToggleGroupItem value="worlds">{t('overview.worlds')}</ToggleGroupItem>
          <ToggleGroupItem value="servers">{t('overview.servers')}</ToggleGroupItem>
        </ToggleGroup>
        <span className="text-sm text-muted-foreground">{summary}</span>
        <Select value={sort} onValueChange={(value) => onSortChange(value as OverviewSort)}>
          <SelectTrigger size="sm" className="ml-auto w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="lastPlayed">{t('toolbar.sortLastPlayed')}</SelectItem>
            <SelectItem value="name">{t('toolbar.sortByName')}</SelectItem>
          </SelectContent>
        </Select>
        <Button size="sm" variant="ghost" onClick={onRefresh}>
          <RefreshCw /> {t('common:action.refresh')}
        </Button>
      </header>
      <ContextMenuArea
        menu={
          <ContextMenuContent className="w-max">
            <ContextMenuItem onSelect={onRefresh}>
              <RefreshCw /> {t('common:action.refresh')}
            </ContextMenuItem>
          </ContextMenuContent>
        }
      >
        <main className="flex min-h-0 flex-1 flex-col gap-5 overflow-auto p-4">{children}</main>
      </ContextMenuArea>
    </div>
  )
}
