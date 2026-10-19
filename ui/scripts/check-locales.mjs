// Fails when a translation misses keys, has extra ones or uses other placeholders than English.
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const root = join(import.meta.dirname, '../src/locales')
const PLURAL = /_(zero|one|two|few|many|other)$/
const languages = readdirSync(root)

const flatten = (object, prefix = '') =>
  Object.entries(object).flatMap(([key, value]) =>
    typeof value === 'object' ? flatten(value, `${prefix}${key}.`) : [[`${prefix}${key}`, value]],
  )

const load = (language, namespace) => {
  const entries = flatten(JSON.parse(readFileSync(join(root, language, `${namespace}.json`), 'utf8')))
  const byKey = new Map()
  for (const [key, value] of entries) {
    const base = key.replace(PLURAL, '')
    const placeholders = new Set([...value.matchAll(/{{\s*(\w+)\s*}}/g)].map((match) => match[1]))
    byKey.set(base, new Set([...(byKey.get(base) ?? []), ...placeholders]))
  }
  return byKey
}

const problems = []
for (const file of readdirSync(join(root, 'en'))) {
  const namespace = file.replace(/\.json$/, '')
  const english = load('en', namespace)
  for (const language of languages.filter((code) => code !== 'en')) {
    let translated
    try {
      translated = load(language, namespace)
    } catch (error) {
      problems.push(`${language}/${file}: ${error.message}`)
      continue
    }
    for (const [key, placeholders] of english) {
      const other = translated.get(key)
      if (!other) problems.push(`${language}/${namespace}: missing ${key}`)
      else if ([...placeholders].some((name) => !other.has(name)))
        problems.push(`${language}/${namespace}: ${key} must use ${[...placeholders].join(', ')}`)
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
