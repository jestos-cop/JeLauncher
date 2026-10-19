import { useEffect } from 'react'
import type { LoaderKind, LoaderSpec } from '@/bindings'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useLoaderVersions } from '@/hooks/use-loader-versions'

const LOADERS: { kind: LoaderKind; label: string }[] = [
  { kind: 'fabric', label: 'Fabric' },
  { kind: 'quilt', label: 'Quilt' },
]

interface LoaderPickerProps {
  minecraftVersion: string | null
  value: LoaderSpec | null
  onChange: (loader: LoaderSpec | null) => void
}

export function LoaderPicker({ minecraftVersion, value, onChange }: LoaderPickerProps) {
  const kind = value?.kind ?? null
  const { data: versions, error, isFetching } = useLoaderVersions(kind, minecraftVersion)

  useEffect(() => {
    if (!kind || !versions) return
    const current = versions.some((v) => v.version === value?.version)
    if (!current) {
      const preferred = versions.find((v) => v.stable) ?? versions[0]
      onChange(preferred ? { kind, version: preferred.version } : null)
    }
  }, [kind, versions, value?.version, onChange])

  const selectKind = (next: string) => {
    if (next === 'none' || !next) onChange(null)
    else onChange({ kind: next as LoaderKind, version: '' })
  }

  return (
    <div className="flex flex-wrap items-center gap-3">
      <Label>Loader</Label>
      <ToggleGroup type="single" variant="outline" size="sm" value={kind ?? 'none'} onValueChange={selectKind}>
        <ToggleGroupItem value="none">None</ToggleGroupItem>
        {LOADERS.map((loader) => (
          <ToggleGroupItem key={loader.kind} value={loader.kind} disabled={!minecraftVersion}>
            {loader.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {kind && (
        <Select
          value={value?.version || undefined}
          onValueChange={(version) => onChange({ kind, version })}
          disabled={isFetching || !versions?.length}
        >
          <SelectTrigger className="w-48" size="sm">
            <SelectValue placeholder={isFetching ? 'Loading…' : 'No versions'} />
          </SelectTrigger>
          <SelectContent>
            {versions?.map((version) => (
              <SelectItem key={version.version} value={version.version}>
                {version.version}
                {!version.stable && <span className="text-muted-foreground">beta</span>}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
      {error && <span className="text-sm text-destructive">{error.message}</span>}
    </div>
  )
}
