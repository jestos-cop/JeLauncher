import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { SessionsProvider } from '@/app/SessionsProvider'
import { MainPage } from '@/pages/main/MainPage'

const queryClient = new QueryClient()

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionsProvider>
        <MainPage />
      </SessionsProvider>
    </QueryClientProvider>
  )
}
