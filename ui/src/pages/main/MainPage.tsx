import { Plus, RefreshCw, Settings } from 'lucide-react'
import { useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { AccountsMenu } from '@/components/AccountsMenu'
import { FoldersMenu } from '@/components/FoldersMenu'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'
import { useInstances } from '@/hooks/use-instances'
import { useStoredState } from '@/hooks/use-stored-state'
import { InstanceGrid, type InstanceSort } from '@/pages/main/InstanceGrid'
import { InstancePanel } from '@/pages/main/InstancePanel'
import { StatusBar } from '@/pages/main/StatusBar'

export function MainPage() {
  const { data: instances, error, isPending, refetch } = useInstances()
  const [isCreating, setIsCreating] = useState(false)
  const { navigate } = useNavigation()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [sort, setSort] = useStoredState<InstanceSort>('instance-sort', 'name')
  const selected = instances?.find((instance) => instance.id === selectedId)

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-2 border-b px-3 py-2">
        <Button size="sm" onClick={() => setIsCreating(true)}>
          <Plus /> Add Instance
        </Button>
        <Button size="sm" variant="ghost" onClick={() => refetch()}>
          <RefreshCw /> Refresh
        </Button>
        <FoldersMenu />
        <Button size="sm" variant="ghost" onClick={() => navigate({ page: 'settings', tab: 'launcher' })}>
          <Settings /> Settings
        </Button>
        <div className="ml-auto flex items-center gap-2">
          <Input
            className="h-8 w-48"
            placeholder="Search instances"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <Select value={sort} onValueChange={(value) => setSort(value as InstanceSort)}>
            <SelectTrigger size="sm" className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="name">By name</SelectItem>
              <SelectItem value="lastPlayed">Last played</SelectItem>
            </SelectContent>
          </Select>
          <AccountsMenu />
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <main className="flex-1 overflow-auto p-4">
          {isPending && <p className="text-muted-foreground">Loading instances…</p>}
          {error && <p className="text-destructive">{error.message}</p>}
          {instances?.length === 0 && <p className="text-muted-foreground">No instances yet.</p>}
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
