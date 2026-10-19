import { useDeferredValue, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { ProjectHit, Provider } from '@/bindings'
import { ProjectRow } from '@/components/ProjectRow'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { type ModpackSelection, useModpackVersions } from '@/hooks/use-modpacks'
import { useResourceSearch } from '@/hooks/use-modplatform'
import { PROVIDER_NAMES } from '@/lib/providers'
import { cn } from '@/lib/utils'

interface ModpackBrowserProps {
  provider: Provider
  selected: ModpackSelection | null
  onSelect: (selection: ModpackSelection | null) => void
}

export function ModpackBrowser({ provider, selected, onSelect }: ModpackBrowserProps) {
  const { t } = useTranslation('newInstance')
  const [text, setText] = useState('')
  const [project, setProject] = useState<ProjectHit | null>(null)
  const search = useResourceSearch({
    provider,
    text: useDeferredValue(text),
    kind: 'modpack',
    sort: 'relevance',
    gameVersion: null,
    loader: null,
  })
  const hits = search.data?.pages.flatMap((page) => page.hits) ?? []

  const selectProject = (hit: ProjectHit) => {
    setProject(hit)
    onSelect(null)
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <Input
        placeholder={t('modpackBrowser.search', { provider: PROVIDER_NAMES[provider] })}
        value={text}
        onChange={(event) => setText(event.target.value)}
      />
      <div className="grid min-h-0 flex-1 grid-cols-[1fr_16rem] gap-3">
        <div className="min-h-0 overflow-auto rounded-md border">
          {search.error && <p className="p-3 text-sm text-destructive">{search.error.message}</p>}
          <ul>
            {hits.map((hit) => (
              <ProjectRow
                key={hit.id}
                hit={hit}
                isSelected={hit.id === project?.id}
                onSelect={() => selectProject(hit)}
              />
            ))}
          </ul>
          {search.hasNextPage && (
            <Button
              variant="ghost"
              className="w-full"
              disabled={search.isFetchingNextPage}
              onClick={() => void search.fetchNextPage()}
            >
              {t('modpackBrowser.loadMore')}
            </Button>
          )}
        </div>
        {project ? (
          <ModpackVersions
            provider={provider}
            projectId={project.id}
            selected={selected?.versionId ?? null}
            onSelect={(versionId) => onSelect({ provider, projectId: project.id, versionId })}
          />
        ) : (
          <p className="p-3 text-sm text-muted-foreground">{t('modpackBrowser.selectModpack')}</p>
        )}
      </div>
    </div>
  )
}

interface ModpackVersionsProps {
  provider: Provider
  projectId: string
  selected: string | null
  onSelect: (versionId: string) => void
}

function ModpackVersions({ provider, projectId, selected, onSelect }: ModpackVersionsProps) {
  const { t } = useTranslation('newInstance')
  const { data: versions, error, isPending } = useModpackVersions(provider, projectId)
  return (
    <div className="min-h-0 overflow-auto rounded-md border">
      {isPending && (
        <p className="p-3 text-sm text-muted-foreground">{t('modpackBrowser.loadingVersions')}</p>
      )}
      {error && <p className="p-3 text-sm text-destructive">{error.message}</p>}
      <ul>
        {versions?.map((version) => (
          <li key={version.id}>
            <button
              type="button"
              onClick={() => onSelect(version.id)}
              className={cn(
                'flex w-full items-center gap-2 p-2 text-left hover:bg-accent',
                version.id === selected && 'bg-accent',
              )}
            >
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{version.versionNumber}</p>
                <p className="truncate text-xs text-muted-foreground">{version.name}</p>
              </div>
              {version.releaseType !== 'release' && (
                <Badge variant="secondary">{t(`releaseType.${version.releaseType}`)}</Badge>
              )}
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
