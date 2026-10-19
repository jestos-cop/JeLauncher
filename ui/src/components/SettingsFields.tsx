import { RefreshCw } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Theme } from '@/bindings'
import { useJavaInstalls } from '@/hooks/use-settings'

const AUTO_JAVA = 'auto'

export function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="grid grid-cols-[12rem_1fr] items-center gap-3">
      <Label>{label}</Label>
      <div className="flex flex-col gap-1">
        {children}
        {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
      </div>
    </div>
  )
}

interface NumberInputProps {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  suffix?: string
}

export function NumberInput({
  value,
  onChange,
  min = 0,
  max = Number.MAX_SAFE_INTEGER,
  step = 1,
  suffix,
}: NumberInputProps) {
  return (
    <div className="flex items-center gap-2">
      <Input
        type="number"
        className="w-32"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Math.min(max, Math.max(min, Number(event.target.value) || 0)))}
      />
      {suffix && <span className="text-sm text-muted-foreground">{suffix}</span>}
    </div>
  )
}

interface JavaPickerProps {
  /** `null` means the launcher picks a compatible Java automatically. */
  value: string | null
  onChange: (path: string | null) => void
}

export function JavaPicker({ value, onChange }: JavaPickerProps) {
  const { data: installs = [], isFetching, refetch } = useJavaInstalls()
  const isUnlisted = value !== null && !installs.some((install) => install.path === value)

  return (
    <div className="flex items-center gap-2">
      <Select value={value ?? AUTO_JAVA} onValueChange={(next) => onChange(next === AUTO_JAVA ? null : next)}>
        <SelectTrigger className="w-full max-w-xl">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={AUTO_JAVA}>Automatic (best match for the game version)</SelectItem>
          {isUnlisted && <SelectItem value={value}>{value}</SelectItem>}
          {installs.map((install) => (
            <SelectItem key={install.path} value={install.path}>
              Java {install.major} · {install.vendor} · {install.arch} — {install.path}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button size="sm" variant="ghost" disabled={isFetching} onClick={() => void refetch()}>
        <RefreshCw /> Detect
      </Button>
    </div>
  )
}

export function ThemeSelect({ value, onChange }: { value: Theme; onChange: (theme: Theme) => void }) {
  return (
    <Select value={value} onValueChange={(theme) => onChange(theme as Theme)}>
      <SelectTrigger className="w-40">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="system">System</SelectItem>
        <SelectItem value="light">Light</SelectItem>
        <SelectItem value="dark">Dark</SelectItem>
      </SelectContent>
    </Select>
  )
}
