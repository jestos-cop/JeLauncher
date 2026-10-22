import type { LoaderKind } from '@/bindings'

/** Short loader names for compact labels. */
export const LOADER_LABELS: Record<LoaderKind, string> = {
  fabric: 'Fabric',
  quilt: 'Quilt',
  forge: 'Forge',
  neoforge: 'NeoForge',
}
