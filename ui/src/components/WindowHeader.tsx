import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

/** On macOS the window has no title bar of its own, so the header leaves room for the window buttons. */
const IS_MACOS = navigator.userAgent.includes('Mac')

/** Top bar of a page; its empty space drags the window. */
export function WindowHeader({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <header
      data-tauri-drag-region
      className={cn('flex items-center border-b px-3 py-2', IS_MACOS && 'pl-24', className)}
    >
      {children}
    </header>
  )
}
