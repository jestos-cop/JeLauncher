import type { ResourceFile } from '@/bindings'

export const STATUSES = ['enabled', 'disabled'] as const
export const SOURCES = ['modrinth', 'curseForge', 'none'] as const
export const TRAITS = ['hasUpdate', 'locked', 'incompatible', 'isDependency', 'hasDependencies'] as const

type Status = (typeof STATUSES)[number]
type SourceFilter = (typeof SOURCES)[number]
type Trait = (typeof TRAITS)[number]

/** Files must match one of the picked values of every group and every picked trait. */
export interface ResourceFilters {
  statuses: Status[]
  sources: SourceFilter[]
  traits: Trait[]
}

export const NO_FILTERS: ResourceFilters = { statuses: [], sources: [], traits: [] }

const hasStatus = (file: ResourceFile, status: Status) => file.enabled === (status === 'enabled')

const hasSource = (file: ResourceFile, source: SourceFilter) =>
  source === 'none' ? file.source === null : file.source?.provider === source

function hasTrait(file: ResourceFile, trait: Trait, hasUpdate: (file: ResourceFile) => boolean) {
  switch (trait) {
    case 'hasUpdate':
      return hasUpdate(file)
    case 'locked':
      return file.source?.lockUpdate === true
    case 'incompatible':
      return file.incompatible
    case 'isDependency':
      return file.requiredBy.length > 0
    case 'hasDependencies':
      return file.requires.length > 0
  }
}

export function matchesFilters(
  file: ResourceFile,
  filters: ResourceFilters,
  hasUpdate: (file: ResourceFile) => boolean,
) {
  return (
    (filters.statuses.length === 0 || filters.statuses.some((status) => hasStatus(file, status))) &&
    (filters.sources.length === 0 || filters.sources.some((source) => hasSource(file, source))) &&
    filters.traits.every((trait) => hasTrait(file, trait, hasUpdate))
  )
}
