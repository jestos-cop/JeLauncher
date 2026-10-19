import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { commands, type GlobalSettings } from '@/bindings'
import { unwrap } from '@/lib/command'

const SETTINGS_KEY = ['settings']

export function useSettings() {
  return useQuery({ queryKey: SETTINGS_KEY, queryFn: () => unwrap(commands.getSettings()) })
}

export function useUpdateSettings() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (settings: GlobalSettings) => unwrap(commands.updateSettings(settings)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SETTINGS_KEY }),
  })
}

export function useJavaInstalls() {
  return useQuery({
    queryKey: ['java-installs'],
    queryFn: () => unwrap(commands.detectJava()),
    staleTime: Infinity,
  })
}

export function useSystemMemory() {
  return useQuery({
    queryKey: ['system-memory'],
    queryFn: () => commands.systemMemoryMb(),
    staleTime: Infinity,
  })
}

/** Hint for a maximum memory field: how much the computer has and whether the value fits. */
export function useMemoryHint(maxMemoryMb: number) {
  const { t } = useTranslation('settings')
  const { data: total } = useSystemMemory()
  if (!total) return undefined
  const gib = (total / 1024).toFixed(1).replace(/\.0$/, '')
  return t(maxMemoryMb > total ? 'java.memoryOverTotal' : 'java.memoryTotal', { gib })
}
