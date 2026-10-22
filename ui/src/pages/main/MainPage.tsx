import { Globe, Plus, RefreshCw, Settings } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigation } from '@/app/navigation'
import { AccountsMenu } from '@/components/AccountsMenu'
import { FoldersMenu } from '@/components/FoldersMenu'
import { HelpMenu } from '@/components/HelpMenu'
import { Logo } from '@/components/Logo'
import { NewsBar } from '@/components/NewsBar'
import { ToolbarLabel } from '@/components/ToolbarLabel'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'
import { useInstances } from '@/hooks/use-instances'
import { useSettings } from '@/hooks/use-settings'
import { useStoredState } from '@/hooks/use-stored-state'
import { InstanceGrid, type InstanceSort } from '@/pages/main/InstanceGrid'
import { InstancePanel } from '@/pages/main/InstancePanel'
import { StatusBar } from '@/pages/main/StatusBar'

export function MainPage() {
  const { t } = useTranslation(['main', 'common'])
  const { data: instances, error, isPending, refetch } = useInstances()
  const [isCreating, setIsCreating] = useState(false)
  const { navigate } = useNavigation()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [sort, setSort] = useStoredState<InstanceSort>('instance-sort', 'name')
  const { data: settings } = useSettings()
  const selected = instances?.find((instance) => instance.id === selectedId)

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex flex-wrap items-center gap-2 border-b px-3 py-2">
        <Logo className="mr-1" />
        <Button size="sm" onClick={() => setIsCreating(true)}>
          <Plus /> {t('toolbar.addInstance')}
        </Button>
        <Button size="sm" variant="ghost" title={t('common:action.refresh')} onClick={() => refetch()}>
          <RefreshCw /> <ToolbarLabel>{t('common:action.refresh')}</ToolbarLabel>
        </Button>
        <FoldersMenu />
        <Button
          size="sm"
          variant="ghost"
          title={t('toolbar.worlds')}
          onClick={() => navigate({ page: 'worlds' })}
        >
          <Globe /> <ToolbarLabel>{t('toolbar.worlds')}</ToolbarLabel>
        </Button>
        <Button
          size="sm"
          variant="ghost"
          title={t('toolbar.settings')}
          onClick={() => navigate({ page: 'settings', tab: 'launcher' })}
        >
          <Settings /> <ToolbarLabel>{t('toolbar.settings')}</ToolbarLabel>
        </Button>
        <HelpMenu />
        <div className="ml-auto flex items-center gap-2">
          <Input
            className="h-8 w-32 lg:w-48"
            placeholder={t('toolbar.searchPlaceholder')}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <Select value={sort} onValueChange={(value) => setSort(value as InstanceSort)}>
            <SelectTrigger size="sm" className="w-32 lg:w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="name">{t('toolbar.sortByName')}</SelectItem>
              <SelectItem value="lastPlayed">{t('toolbar.sortLastPlayed')}</SelectItem>
            </SelectContent>
          </Select>
          <AccountsMenu />
        </div>
      </header>
      {(settings?.appearance.showNewsBar ?? true) && <NewsBar />}

      <div className="flex min-h-0 flex-1">
        <main className="flex-1 overflow-auto p-4">
          {isPending && <p className="text-muted-foreground">{t('grid.loading')}</p>}
          {error && <p className="text-destructive">{error.message}</p>}
          {instances?.length === 0 && <p className="text-muted-foreground">{t('grid.empty')}</p>}
          {instances && instances.length > 0 && (
            <InstanceGrid
              instances={instances}
              search={search}
              sort={sort}
              selectedId={selectedId}
              onSelect={setSelectedId}
            />
          )}
        </main>
        {selected && <InstancePanel instance={selected} />}
      </div>

      <StatusBar instances={instances ?? []} />
      <NewInstanceDialog open={isCreating} onOpenChange={setIsCreating} />
    </div>
  )
}
