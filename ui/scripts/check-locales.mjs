// Fails when a translation misses keys or plural forms, has extra keys or uses other placeholders than English.
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const root = join(import.meta.dirname, '../src/locales')
const PLURAL = /_(zero|one|two|few|many|other)$/
const languages = readdirSync(root)

const flatten = (object, prefix = '') =>
  Object.entries(object).flatMap(([key, value]) =>
    typeof value === 'object' ? flatten(value, `${prefix}${key}.`) : [[`${prefix}${key}`, value]],
  )

/** Base keys with their placeholders and, for plural keys, the plural forms present. */
const load = (language, namespace) => {
  const entries = flatten(JSON.parse(readFileSync(join(root, language, `${namespace}.json`), 'utf8')))
  const byKey = new Map()
  for (const [key, value] of entries) {
    const base = key.replace(PLURAL, '')
    const entry = byKey.get(base) ?? { placeholders: new Set(), forms: new Set() }
    for (const match of value.matchAll(/{{\s*(\w+)\s*}}/g)) entry.placeholders.add(match[1])
    const form = PLURAL.exec(key)
    if (form) entry.forms.add(form[1])
    byKey.set(base, entry)
  }
  return byKey
}

const problems = []
for (const file of readdirSync(join(root, 'en'))) {
  const namespace = file.replace(/\.json$/, '')
  const english = load('en', namespace)
  for (const language of languages.filter((code) => code !== 'en')) {
    const pluralForms = new Intl.PluralRules(language).resolvedOptions().pluralCategories
    let translated
    try {
      translated = load(language, namespace)
    } catch (error) {
      problems.push(`${language}/${file}: ${error.message}`)
      continue
    }
    for (const [key, { placeholders, forms }] of english) {
      const other = translated.get(key)
      if (!other) {
        problems.push(`${language}/${namespace}: missing ${key}`)
        continue
      }
      if ([...placeholders].some((name) => !other.placeholders.has(name)))
        problems.push(`${language}/${namespace}: ${key} must use ${[...placeholders].join(', ')}`)
      const missingForms = forms.size > 0 ? pluralForms.filter((form) => !other.forms.has(form)) : []
      if (missingForms.length > 0)
        problems.push(`${language}/${namespace}: ${key} needs plural forms ${missingForms.join(', ')}`)
    }
    for (const key of translated.keys()) {
      if (!english.has(key)) problems.push(`${language}/${namespace}: unknown key ${key}`)
    }
  }
}
if (problems.length > 0) {
  console.error(problems.join('\n'))
  process.exit(1)
}
