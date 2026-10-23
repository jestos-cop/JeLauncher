import { Users } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { ServerStatus } from '@/bindings'

/** Who is online, when the server shares some of the names. */
export function ServerPlayers({ status }: { status: ServerStatus }) {
  const { t } = useTranslation('instance')
  if (status.players.length === 0) return null
  const hidden = status.playersOnline - status.players.length
  return (
    <p
      className="flex items-center gap-1 truncate text-xs text-muted-foreground"
      title={status.players.join('\n')}
    >
      <Users className="size-3 shrink-0" />
      <span className="truncate">
        {status.players.join(', ')}
        {hidden > 0 && ` ${t('servers.morePlayers', { count: hidden })}`}
      </span>
    </p>
  )
}
