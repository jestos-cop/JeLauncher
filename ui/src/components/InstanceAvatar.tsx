import { convertFileSrc } from '@tauri-apps/api/core'
import type { InstanceSummary } from '@/bindings'
import { useCustomIcons } from '@/hooks/use-icons'
import { BUILTIN_ICONS, DEFAULT_ICON } from '@/lib/builtin-icons'
import { cn } from '@/lib/utils'

const LETTER_COLORS = [
  'from-rose-400 to-rose-600',
  'from-orange-400 to-orange-600',
  'from-amber-400 to-amber-600',
  'from-emerald-400 to-emerald-600',
  'from-teal-400 to-teal-600',
  'from-sky-400 to-sky-600',
  'from-indigo-400 to-indigo-600',
  'from-violet-400 to-violet-600',
  'from-fuchsia-400 to-fuchsia-600',
]

/** The same name always gets the same color. */
function letterColor(name: string) {
  let hash = 0
  for (const char of name) hash = (hash * 31 + char.charCodeAt(0)) | 0
  return LETTER_COLORS[Math.abs(hash) % LETTER_COLORS.length]
}

/** Rounded instance picture: its custom image, its built-in icon, or the first letter of its name. */
export function InstanceAvatar({ instance, className }: { instance: InstanceSummary; className?: string }) {
  const { data: customIcons } = useCustomIcons()
  const custom = customIcons?.find((icon) => icon.key === instance.iconKey)
  const frame = cn('flex shrink-0 items-center justify-center overflow-hidden rounded-xl', className)

  if (custom) {
    return (
      <span className={cn(frame, 'bg-muted')}>
        <img src={convertFileSrc(custom.path)} alt="" className="size-full object-cover" />
      </span>
    )
  }
  const Icon = instance.iconKey !== DEFAULT_ICON ? BUILTIN_ICONS[instance.iconKey] : undefined
  if (Icon) {
    return (
      <span className={cn(frame, 'bg-muted text-muted-foreground')}>
        <Icon className="size-3/5" />
      </span>
    )
  }
  return (
    <span
      className={cn(frame, 'bg-gradient-to-br text-lg font-semibold text-white', letterColor(instance.name))}
    >
      {instance.name.trim().charAt(0).toUpperCase() || '?'}
    </span>
  )
}
