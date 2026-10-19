import { convertFileSrc } from '@tauri-apps/api/core'
import { useCustomIcons } from '@/hooks/use-icons'
import { BUILTIN_ICONS, DEFAULT_ICON } from '@/lib/builtin-icons'
import { cn } from '@/lib/utils'

export function InstanceIcon({ iconKey, className }: { iconKey: string; className?: string }) {
  const { data: customIcons } = useCustomIcons()
  const custom = customIcons?.find((icon) => icon.key === iconKey)
  if (custom) {
    return (
      <img src={convertFileSrc(custom.path)} alt="" className={cn('rounded object-contain', className)} />
    )
  }
  const Icon = BUILTIN_ICONS[iconKey] ?? BUILTIN_ICONS[DEFAULT_ICON]
  return <Icon className={cn('text-muted-foreground', className)} />
}
