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
  'pt-BR': 'Português (Brasil)',
  zh: '简体中文',
  ja: '日本語',
  'en-PT': 'Pirate English',
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

/** A system language tag such as `pt-PT` or `zh-Hans-CN` matched exactly, or by its base language. */
function matchLanguage(tag: string): Language | undefined {
  const exact = Object.keys(LANGUAGES).find((code) => code.toLowerCase() === tag.toLowerCase())
  if (exact && isLanguage(exact)) return exact
  const base = tag.split('-')[0].toLowerCase()
  return (Object.keys(LANGUAGES) as Language[]).find((code) => code.split('-')[0] === base)
}

/** The configured language, or the first system language the launcher speaks, or English. */
export function resolveLanguage(configured: string | null): Language {
  if (configured && isLanguage(configured)) return configured
  return navigator.languages.map(matchLanguage).find((language) => language !== undefined) ?? 'en'
}

void i18n.use(initReactI18next).init({
  resources,
  lng: resolveLanguage(null),
  fallbackLng: 'en',
  defaultNS: 'common',
  interpolation: { escapeValue: false },
})

export { i18n }
