import { cn } from '@/lib/utils'

export function Logo({ className }: { className?: string }) {
  return (
    <img src="/logo.svg" alt="JeLauncher" className={cn('size-7 select-none', className)} draggable={false} />
  )
}
