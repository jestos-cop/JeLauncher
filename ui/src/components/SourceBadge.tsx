import { useTranslation } from 'react-i18next'
import type { ResourceSource } from '@/bindings'
import { PROVIDER_NAMES } from '@/lib/providers'
import { cn } from '@/lib/utils'

const PROVIDER_COLORS: Partial<Record<ResourceSource['provider'], string>> = {
  modrinth: 'bg-[#1bd96a]',
  curseForge: 'bg-[#f16436]',
}

/** The platform an installed resource came from, or a dash when it is unknown. */
export function SourceBadge({ source }: { source: ResourceSource | null }) {
  const { t } = useTranslation('instance')
  if (!source) {
    return (
      <span className="text-muted-foreground" title={t('resources.unknownSource')}>
        —
      </span>
    )
  }
  return (
    <span className="inline-flex items-center gap-1.5 whitespace-nowrap" title={source.name}>
      <span
        className={cn('size-2 rounded-full', PROVIDER_COLORS[source.provider] ?? 'bg-muted-foreground')}
      />
      {PROVIDER_NAMES[source.provider]}
    </span>
  )
}
