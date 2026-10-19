import type { Provider } from '@/bindings'

export const PROVIDER_NAMES: Record<Provider, string> = {
  modrinth: 'Modrinth',
  curseForge: 'CurseForge',
  ftb: 'FTB',
}

/** Platforms that publish single mods and packs, not just modpacks. */
export const RESOURCE_PROVIDERS: Provider[] = ['modrinth', 'curseForge']
