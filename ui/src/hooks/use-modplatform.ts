import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type ResourceKind, type SearchQuery } from '@/bindings'
import { resourcesKey } from '@/hooks/use-resources'
import { unwrap } from '@/lib/command'

const PAGE_SIZE = 20

export function useResourceSearch(query: Omit<SearchQuery, 'offset' | 'limit'>) {
  return useInfiniteQuery({
    queryKey: ['resource-search', query],
    queryFn: ({ pageParam }) =>
      unwrap(commands.searchResources({ ...query, offset: pageParam, limit: PAGE_SIZE })),
    initialPageParam: 0,
    getNextPageParam: (lastPage, pages) => {
      const loaded = pages.reduce((count, page) => count + page.hits.length, 0)
      return loaded < lastPage.total ? loaded : undefined
    },
  })
}

export function useProjectVersions(instanceId: string, projectId: string | null, kind: ResourceKind) {
  return useQuery({
    queryKey: ['project-versions', instanceId, projectId, kind],
    queryFn: () => unwrap(commands.listProjectVersions(instanceId, projectId!, kind)),
    enabled: projectId !== null,
  })
}

export function useInstallVersion(instanceId: string, kind: ResourceKind) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (versionId: string) => unwrap(commands.installProjectVersion(instanceId, kind, versionId)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) }),
  })
}
