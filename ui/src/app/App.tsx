import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ConsoleBehavior } from '@/app/ConsoleBehavior'
import { ThemeSync } from '@/app/ThemeSync'
import { useNavigation } from '@/app/navigation'
import { NavigationProvider } from '@/app/NavigationProvider'
import { SessionsProvider } from '@/app/SessionsProvider'
import { SetupWizard } from '@/dialogs/SetupWizard'
import { useSettings } from '@/hooks/use-settings'
import { InstancePage } from '@/pages/instance/InstancePage'
import { MainPage } from '@/pages/main/MainPage'
import { SettingsPage } from '@/pages/settings/SettingsPage'

const queryClient = new QueryClient()

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionsProvider>
        <NavigationProvider>
          <ConsoleBehavior />
          <ThemeSync />
          <CurrentPage />
          <FirstRunSetup />
        </NavigationProvider>
      </SessionsProvider>
    </QueryClientProvider>
  )
}

function FirstRunSetup() {
  const { data: settings } = useSettings()
  return settings && !settings.launcher.setupCompleted ? <SetupWizard saved={settings} /> : null
}

function CurrentPage() {
  const { view } = useNavigation()
  switch (view.page) {
    case 'main':
      return <MainPage />
    case 'instance':
      return <InstancePage instanceId={view.instanceId} tab={view.tab} />
    case 'settings':
      return <SettingsPage tab={view.tab} />
  }
}
