import { useQuery } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useMinecraftVersions() {
  return useQuery({
    queryKey: ['minecraft-versions'],
    queryFn: () => unwrap(commands.listMinecraftVersions()),
    staleTime: 10 * 60 * 1000,
  })
}
