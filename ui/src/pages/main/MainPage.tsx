import { Plus, RefreshCw, Settings } from 'lucide-react'
import { useState } from 'react'
import { useNavigation } from '@/app/navigation'
import { AccountsMenu } from '@/components/AccountsMenu'
import { Button } from '@/components/ui/button'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'
import { useInstances } from '@/hooks/use-instances'
import { InstanceGrid } from '@/pages/main/InstanceGrid'
import { InstancePanel } from '@/pages/main/InstancePanel'

export function MainPage() {
  const { data: instances, error, isPending, refetch } = useInstances()
  const [isCreating, setIsCreating] = useState(false)
  const { navigate } = useNavigation()
  const [selectedId, setSelectedId] = useState<string | null>(null)
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
        <Button size="sm" variant="ghost" onClick={() => navigate({ page: 'settings', tab: 'java' })}>
          <Settings /> Settings
        </Button>
        <div className="ml-auto">
          <AccountsMenu />
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <main className="flex-1 overflow-auto p-4">
          {isPending && <p className="text-muted-foreground">Loading instances…</p>}
          {error && <p className="text-destructive">{error.message}</p>}
          {instances?.length === 0 && <p className="text-muted-foreground">No instances yet.</p>}
          {instances && instances.length > 0 && (
            <InstanceGrid instances={instances} selectedId={selectedId} onSelect={setSelectedId} />
          )}
        </main>
        {selected && <InstancePanel instance={selected} />}
      </div>

      <NewInstanceDialog open={isCreating} onOpenChange={setIsCreating} />
    </div>
  )
}
