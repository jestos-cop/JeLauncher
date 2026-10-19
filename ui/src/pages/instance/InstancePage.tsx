import {
  Camera,
  Database,
  Globe,
  Image,
  Layers,
  NotebookPen,
  Package,
  Puzzle,
  ScrollText,
  Server,
  SlidersHorizontal,
  Sparkles,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useNavigation, type InstanceTab } from '@/app/navigation'
import { useSession } from '@/app/sessions'
import { LaunchControls } from '@/components/LaunchControls'
import { LogView } from '@/components/LogView'
import { SidebarLayout, type SidebarTab } from '@/components/SidebarLayout'
import { useInstances } from '@/hooks/use-instances'
import { InstanceSettingsPage } from '@/pages/instance/InstanceSettingsPage'
import { ModpackPage } from '@/pages/instance/ModpackPage'
import { NotesPage } from '@/pages/instance/NotesPage'
import { ResourcesPage } from '@/pages/instance/ResourcesPage'
import { ScreenshotsPage } from '@/pages/instance/ScreenshotsPage'
import { ServersPage } from '@/pages/instance/ServersPage'
import { VersionPage } from '@/pages/instance/VersionPage'
import { WorldsPage } from '@/pages/instance/WorldsPage'

const TABS: Omit<SidebarTab<InstanceTab>, 'label'>[] = [
  { tab: 'log', icon: ScrollText },
  { tab: 'modpack', icon: Package },
  { tab: 'version', icon: Layers },
  { tab: 'mod', icon: Puzzle },
  { tab: 'resourcePack', icon: Image },
  { tab: 'shaderPack', icon: Sparkles },
  { tab: 'dataPack', icon: Database },
  { tab: 'worlds', icon: Globe },
  { tab: 'servers', icon: Server },
  { tab: 'screenshots', icon: Camera },
  { tab: 'notes', icon: NotebookPen },
  { tab: 'settings', icon: SlidersHorizontal },
]

interface InstancePageProps {
  instanceId: string
  tab: InstanceTab
}

export function InstancePage({ instanceId, tab }: InstancePageProps) {
  const { t } = useTranslation('instance')
  const { navigate } = useNavigation()
  const { data: instances } = useInstances()
  const instance = instances?.find((candidate) => candidate.id === instanceId)
  const tabs = TABS.filter((entry) => entry.tab !== 'modpack' || instance?.managedPack).map((entry) => ({
    ...entry,
    label: t(`tabs.${entry.tab}`),
  }))

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
      tabs={tabs}
      current={tab}
      onTabChange={(next) => navigate({ page: 'instance', instanceId, tab: next })}
      onBack={() => navigate({ page: 'main' })}
    >
      {tab === 'log' && <InstanceLog instanceId={instanceId} />}
      {tab === 'modpack' && instance?.managedPack && (
        <ModpackPage instance={instance} pack={instance.managedPack} />
      )}
      {tab === 'version' && instance && <VersionPage instance={instance} />}
      {(tab === 'mod' || tab === 'resourcePack' || tab === 'shaderPack' || tab === 'dataPack') &&
        instance && <ResourcesPage key={tab} instance={instance} kind={tab} />}
      {tab === 'worlds' && <WorldsPage instanceId={instanceId} />}
      {tab === 'servers' && <ServersPage instanceId={instanceId} />}
      {tab === 'screenshots' && <ScreenshotsPage instanceId={instanceId} />}
      {tab === 'notes' && <NotesPage key={instanceId} instanceId={instanceId} />}
      {tab === 'settings' && <InstanceSettingsPage instanceId={instanceId} />}
    </SidebarLayout>
  )
}

function InstanceLog({ instanceId }: { instanceId: string }) {
  const { session, clearLog } = useSession(instanceId)
  return <LogView lines={session.log} onClear={clearLog} />
}
