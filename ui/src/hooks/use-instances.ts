import { useQuery } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useInstances() {
  return useQuery({
    queryKey: ['instances'],
    queryFn: () => unwrap(commands.listInstances()),
  })
}
