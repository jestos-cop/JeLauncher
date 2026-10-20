import { ChevronDown, CircleHelp, Code, FileText, Info } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { commands } from '@/bindings'
import { Button } from '@/components/ui/button'
import { ToolbarLabel } from '@/components/ToolbarLabel'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { AboutDialog } from '@/dialogs/AboutDialog'
import { unwrap } from '@/lib/command'

export function HelpMenu() {
  const { t } = useTranslation('main')
  const [isAboutOpen, setIsAboutOpen] = useState(false)
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm" variant="ghost" title={t('help.title')}>
            <CircleHelp /> <ToolbarLabel>{t('help.title')}</ToolbarLabel> <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem onSelect={() => void unwrap(commands.openRepository())}>
            <Code /> {t('help.sourceCode')}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => void unwrap(commands.openLauncherLog())}>
            <FileText /> {t('help.launcherLog')}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setIsAboutOpen(true)}>
            <Info /> {t('help.about')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {isAboutOpen && <AboutDialog onClose={() => setIsAboutOpen(false)} />}
    </>
  )
}
