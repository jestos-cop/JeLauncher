import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type ProjectHit, type ResourceKind, type SearchQuery } from '@/bindings'
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

export function useProjectVersions(instanceId: string, project: ProjectHit, kind: ResourceKind) {
  return useQuery({
    queryKey: ['project-versions', instanceId, project.provider, project.id, kind],
    queryFn: () => unwrap(commands.listProjectVersions(instanceId, project.provider, project.id, kind)),
  })
}

export function useInstallVersion(instanceId: string, kind: ResourceKind, project: ProjectHit) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (versionId: string) =>
      unwrap(commands.installProjectVersion(instanceId, kind, project.provider, project.id, versionId)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: resourcesKey(instanceId, kind) }),
  })
}
