import { useEffect } from 'react'
import { useSettings } from '@/hooks/use-settings'

const DARK_QUERY = '(prefers-color-scheme: dark)'

/** Applies the configured theme to the document, following the OS when set to "system". */
export function ThemeSync() {
  const { data: settings } = useSettings()
  const theme = settings?.appearance.theme ?? 'system'

  useEffect(() => {
    const media = window.matchMedia(DARK_QUERY)
    const apply = () => {
      const dark = theme === 'dark' || (theme === 'system' && media.matches)
      document.documentElement.classList.toggle('dark', dark)
    }
    apply()
    media.addEventListener('change', apply)
    return () => media.removeEventListener('change', apply)
  }, [theme])

  return null
}
