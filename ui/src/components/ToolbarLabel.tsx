import type { ReactNode } from 'react'

/** Toolbar button text that narrow windows hide, leaving the icon and the button's tooltip. */
export function ToolbarLabel({ children }: { children: ReactNode }) {
  return <span className="hidden md:inline">{children}</span>
}
