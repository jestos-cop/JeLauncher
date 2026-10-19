import type { CommandError, Result } from '@/bindings'

export async function unwrap<T>(result: Promise<Result<T, CommandError>>): Promise<T> {
  const settled = await result
  if (settled.status === 'error') throw new Error(settled.error.message)
  return settled.data
}
