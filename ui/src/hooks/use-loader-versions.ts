import { useQuery } from '@tanstack/react-query'
import { commands, type LoaderKind } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useLoaderVersions(kind: LoaderKind | null, minecraftVersion: string | null) {
  return useQuery({
    queryKey: ['loader-versions', kind, minecraftVersion],
    queryFn: () => unwrap(commands.listLoaderVersions(kind!, minecraftVersion!)),
    enabled: kind !== null && minecraftVersion !== null,
    staleTime: 10 * 60 * 1000,
  })
}
