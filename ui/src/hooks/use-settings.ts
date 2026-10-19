import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
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
