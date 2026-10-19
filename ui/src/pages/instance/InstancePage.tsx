import { ArrowLeft, Puzzle, ScrollText, type LucideIcon } from 'lucide-react'
import { useNavigation, type InstanceTab } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import { LaunchControls } from '@/components/LaunchControls'
import { LogView } from '@/components/LogView'
import { Button } from '@/components/ui/button'
import { useInstances } from '@/hooks/use-instances'
import { cn } from '@/lib/utils'
import { ModsPage } from '@/pages/instance/ModsPage'

const TABS: { tab: InstanceTab; label: string; icon: LucideIcon }[] = [
  { tab: 'log', label: 'Minecraft Log', icon: ScrollText },
  { tab: 'mods', label: 'Mods', icon: Puzzle },
]

interface InstancePageProps {
  instanceId: string
  tab: InstanceTab
}

export function InstancePage({ instanceId, tab }: InstancePageProps) {
  const { navigate } = useNavigation()
  const { data: instances } = useInstances()
  const instance = instances?.find((candidate) => candidate.id === instanceId)
  const goHome = () => navigate({ page: 'main' })

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-3 border-b px-3 py-2">
        <Button size="sm" variant="ghost" onClick={goHome}>
          <ArrowLeft /> Back
        </Button>
        <h1 className="font-semibold">{instance?.name ?? instanceId}</h1>
        {instance && (
          <span className="text-sm text-muted-foreground">Minecraft {instance.minecraftVersion}</span>
        )}
        <div className="ml-auto w-40">
          <LaunchControls instanceId={instanceId} />
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <nav className="flex w-48 shrink-0 flex-col gap-1 border-r p-2">
          {TABS.map(({ tab: id, label, icon: Icon }) => (
            <button
              key={id}
              type="button"
              onClick={() => navigate({ page: 'instance', instanceId, tab: id })}
              className={cn(
                'flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent',
                id === tab && 'bg-accent font-medium',
              )}
            >
              <Icon className="size-4" /> {label}
            </button>
          ))}
        </nav>
        <main className="flex min-h-0 flex-1 flex-col p-4">
          {tab === 'log' && <InstanceLog instanceId={instanceId} />}
          {tab === 'mods' && instance && <ModsPage instance={instance} />}
        </main>
      </div>
    </div>
  )
}

function InstanceLog({ instanceId }: { instanceId: string }) {
  const { session, clearLog } = useSession(instanceId)
  return <LogView lines={session.log} onClear={clearLog} />
}
