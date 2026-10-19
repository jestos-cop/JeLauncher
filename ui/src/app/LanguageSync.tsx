import { useEffect } from 'react'
import { commands } from '@/bindings'
import { useSettings } from '@/hooks/use-settings'
import { i18n, resolveLanguage } from '@/i18n'

/** Switches the interface to the configured language, following the system when unset. */
export function LanguageSync() {
  const { data: settings } = useSettings()
  const language = resolveLanguage(settings?.appearance.language ?? null)

  useEffect(() => {
    void i18n.changeLanguage(language)
    document.documentElement.lang = language
    void commands.setMenuLanguage(language)
  }, [language])

  return null
}
