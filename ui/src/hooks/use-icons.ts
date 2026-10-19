import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'
import { unwrap } from '@/lib/command'

const iconsKey = ['custom-icons']

export function useCustomIcons() {
  return useQuery({ queryKey: iconsKey, queryFn: () => unwrap(commands.listCustomIcons()) })
}

export function useAddCustomIcon() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (source: string) => unwrap(commands.addCustomIcon(source)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: iconsKey }),
  })
}

export function useRemoveCustomIcon() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (key: string) => unwrap(commands.removeCustomIcon(key)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: iconsKey }),
  })
}

export function useSetInstanceIcon() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, iconKey }: { id: string; iconKey: string }) =>
      unwrap(commands.setInstanceIcon(id, iconKey)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}
