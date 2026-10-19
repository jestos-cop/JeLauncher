import { useMemo, useState, type ReactNode } from 'react'
import { NavigationContext, type View } from '@/app/navigation'

export function NavigationProvider({ children }: { children: ReactNode }) {
  const [view, navigate] = useState<View>({ page: 'main' })
  const value = useMemo(() => ({ view, navigate }), [view])
  return <NavigationContext value={value}>{children}</NavigationContext>
}
