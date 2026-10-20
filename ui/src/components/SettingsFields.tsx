import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, RefreshCw } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Theme } from '@/bindings'
import { useJavaInstalls } from '@/hooks/use-settings'
import { LANGUAGES } from '@/i18n'

const AUTO_JAVA = 'auto'

export function Field({ label, hint, children }: { label: string; hint?: ReactNode; children: ReactNode }) {
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

/** A titled card that groups related settings. */
export function SettingsSection({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-4 rounded-lg border bg-card p-4">
      {title && (
        <h3 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">{title}</h3>
      )}
      {children}
    </section>
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
  const { t } = useTranslation('settings')
  const { data: installs = [], isFetching, refetch } = useJavaInstalls()
  const isUnlisted = value !== null && !installs.some((install) => install.path === value)

  return (
    <div className="flex items-center gap-2">
      <Select value={value ?? AUTO_JAVA} onValueChange={(next) => onChange(next === AUTO_JAVA ? null : next)}>
        <SelectTrigger className="w-full max-w-xl">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={AUTO_JAVA}>{t('java.automatic')}</SelectItem>
          {isUnlisted && <SelectItem value={value}>{value}</SelectItem>}
          {installs.map((install) => (
            <SelectItem key={install.path} value={install.path}>
              Java {install.major} · {install.vendor} · {install.arch} — {install.path}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button size="sm" variant="ghost" disabled={isFetching} onClick={() => void refetch()}>
        <RefreshCw /> {t('java.detect')}
      </Button>
    </div>
  )
}

const THEMES: Theme[] = ['system', 'light', 'dark']
const SYSTEM_LANGUAGE = 'system'

export function ThemeSelect({ value, onChange }: { value: Theme; onChange: (theme: Theme) => void }) {
  const { t } = useTranslation()
  return (
    <Select value={value} onValueChange={(theme) => onChange(theme as Theme)}>
      <SelectTrigger className="w-40">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {THEMES.map((theme) => (
          <SelectItem key={theme} value={theme}>
            {t(`theme.${theme}`)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

/** Language picker; `null` follows the system language. */
export function LanguageSelect({
  value,
  onChange,
}: {
  value: string | null
  onChange: (language: string | null) => void
}) {
  const { t } = useTranslation()
  return (
    <Select
      value={value ?? SYSTEM_LANGUAGE}
      onValueChange={(language) => onChange(language === SYSTEM_LANGUAGE ? null : language)}
    >
      <SelectTrigger className="w-48">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={SYSTEM_LANGUAGE}>{t('language.system')}</SelectItem>
        {Object.entries(LANGUAGES).map(([code, name]) => (
          <SelectItem key={code} value={code}>
            {name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

interface PathInputProps {
  value: string | null
  onChange: (path: string | null) => void
  placeholder: string
  directory?: boolean
}

/** A path typed by hand or picked with the system file dialog; empty means unset. */
export function PathInput({ value, onChange, placeholder, directory = false }: PathInputProps) {
  const { t } = useTranslation()
  const browse = async () => {
    const picked = await open({ directory, multiple: false })
    if (typeof picked === 'string') onChange(picked)
  }
  return (
    <div className="flex gap-2">
      <Input
        placeholder={placeholder}
        value={value ?? ''}
        onChange={(event) => onChange(event.target.value || null)}
      />
      <Button variant="outline" onClick={() => void browse()}>
        <FolderOpen /> {t('action.browse')}
      </Button>
    </div>
  )
}
