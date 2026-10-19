import { open } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface ImportSourceProps {
  value: string
  onChange: (value: string) => void
}

export function ImportSource({ value, onChange }: ImportSourceProps) {
  const browse = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Instance archives', extensions: ['mrpack', 'zip'] }],
    })
    if (path) onChange(path)
  }

  return (
    <div className="flex flex-1 flex-col gap-2">
      <p className="text-sm text-muted-foreground">
        Import a Modrinth pack (.mrpack) or an exported instance (.zip) from a file or a URL.
      </p>
      <div className="flex gap-2">
        <Input
          placeholder="File path or https:// URL"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
        <Button variant="outline" onClick={() => void browse()}>
          Browse…
        </Button>
      </div>
    </div>
  )
}
