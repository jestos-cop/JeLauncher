import type { CommandError, ErrorKind, Result } from '@/bindings'

/** A failed command, keeping its kind so callers can react to cancellations and trust prompts. */
export class CommandFailure extends Error {
  readonly kind: ErrorKind

  constructor(error: CommandError) {
    super(error.message)
    this.kind = error.kind
  }
}

export async function unwrap<T>(result: Promise<Result<T, CommandError>>): Promise<T> {
  const settled = await result
  if (settled.status === 'error') throw new CommandFailure(settled.error)
  return settled.data
}

/** The error worth showing to the user, ignoring cancellations and pending trust prompts. */
export function failure(error: Error | null): Error | null {
  if (error instanceof CommandFailure && error.kind.type !== 'failed') return null
  return error
}

/** Hosts the user has to allow before a modpack install can continue. */
export function untrustedHosts(error: Error | null): string[] | null {
  return error instanceof CommandFailure && error.kind.type === 'untrustedHosts' ? error.kind.hosts : null
}
