import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type NewInstance } from '@/bindings'
import { unwrap } from '@/lib/command'

export function useCreateInstance() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (newInstance: NewInstance) => unwrap(commands.createInstance(newInstance)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
  })
}
