import { useQuery } from '@tanstack/react-query'
import { Code } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { commands } from '@/bindings'
import { DiscordIcon } from '@/components/DiscordIcon'
import { Logo } from '@/components/Logo'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { unwrap } from '@/lib/command'

export function AboutDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation('about')
  const { data: info } = useQuery({ queryKey: ['app-info'], queryFn: () => commands.appInfo() })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader className="items-center text-center">
          <Logo className="size-16" />
          <DialogTitle>JeLauncher</DialogTitle>
          <DialogDescription>{t('about.description')}</DialogDescription>
        </DialogHeader>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <dt className="text-muted-foreground">{t('about.version')}</dt>
          <dd>{info?.version}</dd>
          <dt className="text-muted-foreground">{t('about.license')}</dt>
          <dd>{info?.license}</dd>
        </dl>
        <p className="text-xs text-muted-foreground">{t('about.trademark')}</p>
        <div className="grid grid-cols-2 gap-2">
          <Button variant="outline" onClick={() => void unwrap(commands.openRepository())}>
            <Code /> {t('about.sourceCode')}
          </Button>
          <Button variant="outline" onClick={() => void unwrap(commands.openDiscord())}>
            <DiscordIcon /> {t('about.discord')}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
