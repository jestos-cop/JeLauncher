import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

export const LANGUAGES = {
  en: 'English',
  uk: 'Українська',
  ru: 'Русский',
  es: 'Español',
  fr: 'Français',
  pl: 'Polski',
  sr: 'Српски',
} as const

export type Language = keyof typeof LANGUAGES

type Messages = Record<string, unknown>

const files = import.meta.glob<Messages>('../locales/*/*.json', { eager: true, import: 'default' })
const resources: Record<string, Record<string, Messages>> = {}
for (const [path, messages] of Object.entries(files)) {
  const match = /locales\/([^/]+)\/([^/]+)\.json$/.exec(path)
  if (!match) continue
  const [, language, namespace] = match
  resources[language] = { ...resources[language], [namespace]: messages }
}

const isLanguage = (code: string): code is Language => code in LANGUAGES

/** The configured language, or the first system language the launcher speaks, or English. */
export function resolveLanguage(configured: string | null): Language {
  if (configured && isLanguage(configured)) return configured
  const system = navigator.languages.map((code) => code.split('-')[0].toLowerCase())
  return system.find(isLanguage) ?? 'en'
}

void i18n.use(initReactI18next).init({
  resources,
  lng: resolveLanguage(null),
  fallbackLng: 'en',
  defaultNS: 'common',
  interpolation: { escapeValue: false },
})

export { i18n }
