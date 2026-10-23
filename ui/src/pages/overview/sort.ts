import type { InstanceSummary } from '@/bindings'

export type OverviewSort = 'lastPlayed' | 'name'

export const byName = (a: string, b: string) => a.localeCompare(b, undefined, { numeric: true })

/** Groups ordered by `recency` (newest first) or by instance name, with their items sorted the same way. */
export function sortGroups<T>(
  groups: { instance: InstanceSummary; items: T[] }[],
  sort: OverviewSort,
  item: { name: (item: T) => string; recency: (item: T) => number },
  recency: (group: { instance: InstanceSummary; items: T[] }) => number,
) {
  return groups
    .map((group) => ({
      ...group,
      items: [...group.items].sort((a, b) =>
        sort === 'name' ? byName(item.name(a), item.name(b)) : item.recency(b) - item.recency(a),
      ),
    }))
    .sort((a, b) => (sort === 'name' ? byName(a.instance.name, b.instance.name) : recency(b) - recency(a)))
}
