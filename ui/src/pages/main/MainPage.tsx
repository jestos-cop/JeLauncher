import { Box, Plus, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import type { InstanceSummary } from '@/bindings'
import { AccountsMenu } from '@/components/AccountsMenu'
import { Button } from '@/components/ui/button'
import { NewInstanceDialog } from '@/dialogs/NewInstanceDialog'
import { useInstances } from '@/hooks/use-instances'

export function MainPage() {
  const { data: instances, error, isPending, refetch } = useInstances()
  const [isCreating, setIsCreating] = useState(false)

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-2 border-b px-3 py-2">
        <Button size="sm" onClick={() => setIsCreating(true)}>
          <Plus /> Add Instance
        </Button>
        <Button size="sm" variant="ghost" onClick={() => refetch()}>
          <RefreshCw /> Refresh
        </Button>
        <div className="ml-auto">
          <AccountsMenu />
        </div>
      </header>

      <main className="flex-1 overflow-auto p-4">
        {isPending && <p className="text-muted-foreground">Loading instances…</p>}
        {error && <p className="text-destructive">{error.message}</p>}
        {instances?.length === 0 && (
          <p className="text-muted-foreground">No instances yet.</p>
        )}
        {instances && instances.length > 0 && <InstanceGrid instances={instances} />}
      </main>

      <NewInstanceDialog open={isCreating} onOpenChange={setIsCreating} />
    </div>
  )
}

function InstanceGrid({ instances }: { instances: InstanceSummary[] }) {
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-3">
      {instances.map((instance) => (
        <li
          key={instance.id}
          className="flex flex-col items-center gap-2 rounded-md p-3 text-center hover:bg-accent"
        >
          <Box className="size-12 text-muted-foreground" />
          <span className="line-clamp-2 text-sm">{instance.name}</span>
        </li>
      ))}
    </ul>
  )
}
