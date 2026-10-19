import { createContext, useContext } from 'react'

export type InstanceTab = 'log' | 'mod' | 'resourcePack' | 'shaderPack' | 'dataPack' | 'worlds' | 'settings'

export type SettingsTab = 'java' | 'minecraft' | 'commands'

export type View =
  | { page: 'main' }
  | { page: 'instance'; instanceId: string; tab: InstanceTab }
  | { page: 'settings'; tab: SettingsTab }

interface NavigationContextValue {
  view: View
  navigate: (view: View) => void
}

export const NavigationContext = createContext<NavigationContextValue | null>(null)

export function useNavigation() {
  const context = useContext(NavigationContext)
  if (!context) throw new Error('useNavigation must be used inside NavigationProvider')
  return context
}
