import type { CommandError, Result } from '@/bindings'

/** Thrown when the user stopped the task; callers do not report it as a failure. */
export class CancelledError extends Error {}

export async function unwrap<T>(result: Promise<Result<T, CommandError>>): Promise<T> {
  const settled = await result
  if (settled.status === 'error') {
    throw settled.error.cancelled
      ? new CancelledError(settled.error.message)
      : new Error(settled.error.message)
  }
  return settled.data
}

/** The error worth showing to the user, ignoring cancellations. */
export function failure(error: Error | null): Error | null {
  return error instanceof CancelledError ? null : error
}
