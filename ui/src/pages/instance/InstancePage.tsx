import { Puzzle, ScrollText, SlidersHorizontal } from 'lucide-react'
import { useNavigation, type InstanceTab } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import { LaunchControls } from '@/components/LaunchControls'
import { LogView } from '@/components/LogView'
import { SidebarLayout, type SidebarTab } from '@/components/SidebarLayout'
import { useInstances } from '@/hooks/use-instances'
import { InstanceSettingsPage } from '@/pages/instance/InstanceSettingsPage'
import { ModsPage } from '@/pages/instance/ModsPage'

const TABS: SidebarTab<InstanceTab>[] = [
  { tab: 'log', label: 'Minecraft Log', icon: ScrollText },
  { tab: 'mods', label: 'Mods', icon: Puzzle },
  { tab: 'settings', label: 'Settings', icon: SlidersHorizontal },
]

interface InstancePageProps {
  instanceId: string
  tab: InstanceTab
}

export function InstancePage({ instanceId, tab }: InstancePageProps) {
  const { navigate } = useNavigation()
  const { data: instances } = useInstances()
  const instance = instances?.find((candidate) => candidate.id === instanceId)

  return (
    <SidebarLayout
      title={instance?.name ?? instanceId}
      subtitle={
        instance &&
        `Minecraft ${instance.minecraftVersion}${instance.loader ? ` · ${instance.loader.kind} ${instance.loader.version}` : ''}`
      }
      actions={
        <div className="w-40">
          <LaunchControls instanceId={instanceId} />
        </div>
      }
      tabs={TABS}
      current={tab}
      onTabChange={(next) => navigate({ page: 'instance', instanceId, tab: next })}
      onBack={() => navigate({ page: 'main' })}
    >
      {tab === 'log' && <InstanceLog instanceId={instanceId} />}
      {tab === 'mods' && instance && <ModsPage instance={instance} />}
      {tab === 'settings' && <InstanceSettingsPage instanceId={instanceId} />}
    </SidebarLayout>
  )
}

function InstanceLog({ instanceId }: { instanceId: string }) {
  const { session, clearLog } = useSession(instanceId)
  return <LogView lines={session.log} onClear={clearLog} />
}
