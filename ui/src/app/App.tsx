import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useNavigation } from '@/app/navigation'
import { NavigationProvider } from '@/app/NavigationProvider'
import { SessionsProvider } from '@/app/SessionsProvider'
import { InstancePage } from '@/pages/instance/InstancePage'
import { MainPage } from '@/pages/main/MainPage'

const queryClient = new QueryClient()

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionsProvider>
        <NavigationProvider>
          <CurrentPage />
        </NavigationProvider>
      </SessionsProvider>
    </QueryClientProvider>
  )
}

function CurrentPage() {
  const { view } = useNavigation()
  switch (view.page) {
    case 'main':
      return <MainPage />
    case 'instance':
      return <InstancePage instanceId={view.instanceId} tab={view.tab} />
  }
}
