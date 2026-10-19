import { useQuery } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useNews() {
  return useQuery({
    queryKey: ['news'],
    queryFn: () => unwrap(commands.listNews()),
    staleTime: 60 * 60 * 1000,
  })
}
