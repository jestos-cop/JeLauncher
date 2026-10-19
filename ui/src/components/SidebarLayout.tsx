import { ArrowLeft, type LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export interface SidebarTab<T extends string> {
  tab: T
  label: string
  icon: LucideIcon
}

interface SidebarLayoutProps<T extends string> {
  title: ReactNode
  subtitle?: ReactNode
  actions?: ReactNode
  tabs: SidebarTab<T>[]
  current: T
  onTabChange: (tab: T) => void
  onBack: () => void
  children: ReactNode
}

export function SidebarLayout<T extends string>({
  title,
  subtitle,
  actions,
  tabs,
  current,
  onTabChange,
  onBack,
  children,
}: SidebarLayoutProps<T>) {
  const { t } = useTranslation()
  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex items-center gap-3 border-b px-3 py-2">
        <Button size="sm" variant="ghost" onClick={onBack}>
          <ArrowLeft /> {t('action.back')}
        </Button>
        <h1 className="font-semibold">{title}</h1>
        {subtitle && <span className="text-sm text-muted-foreground">{subtitle}</span>}
        {actions && <div className="ml-auto">{actions}</div>}
      </header>

      <div className="flex min-h-0 flex-1">
        <nav className="flex w-48 shrink-0 flex-col gap-1 border-r p-2">
          {tabs.map(({ tab, label, icon: Icon }) => (
            <button
              key={tab}
              type="button"
              onClick={() => onTabChange(tab)}
              className={cn(
                'flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent',
                tab === current && 'bg-accent font-medium',
              )}
            >
              <Icon className="size-4" /> {label}
            </button>
          ))}
        </nav>
        <main className="flex min-h-0 flex-1 flex-col overflow-auto p-4">{children}</main>
      </div>
    </div>
  )
}
