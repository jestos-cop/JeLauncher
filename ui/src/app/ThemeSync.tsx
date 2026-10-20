import { getCurrentWindow } from '@tauri-apps/api/window'
import { useEffect } from 'react'
import { useSettings } from '@/hooks/use-settings'

const DARK_QUERY = '(prefers-color-scheme: dark)'

/** The page background as RGB; the CSS value may use a color space the window API does not parse. */
function backgroundRgb(): [number, number, number] {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = 1
  const context = canvas.getContext('2d')
  if (!context) return [0, 0, 0]
  context.fillStyle = getComputedStyle(document.body).backgroundColor
  context.fillRect(0, 0, 1, 1)
  const [red, green, blue] = context.getImageData(0, 0, 1, 1).data
  return [red, green, blue]
}

/** Applies the configured theme to the document and the window title bar, following the OS for "system". */
export function ThemeSync() {
  const { data: settings } = useSettings()
  const theme = settings?.appearance.theme ?? 'system'

  useEffect(() => {
    const media = window.matchMedia(DARK_QUERY)
    const apply = () => {
      const dark = theme === 'dark' || (theme === 'system' && media.matches)
      document.documentElement.classList.toggle('dark', dark)
      const appWindow = getCurrentWindow()
      void appWindow.setTheme(dark ? 'dark' : 'light')
      void appWindow.setBackgroundColor(backgroundRgb())
    }
    apply()
    media.addEventListener('change', apply)
    return () => media.removeEventListener('change', apply)
  }, [theme])

  return null
}
