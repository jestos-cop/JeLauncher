import type { ReactElement, ReactNode } from 'react'
import { ContextMenu, ContextMenuTrigger } from '@/components/ui/context-menu'

/** Opens `menu` on right click inside `children`; an area nested in another keeps its own menu. */
export function ContextMenuArea({ menu, children }: { menu: ReactNode; children: ReactElement }) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild onContextMenu={(event) => event.stopPropagation()}>
        {children}
      </ContextMenuTrigger>
      {menu}
    </ContextMenu>
  )
}
