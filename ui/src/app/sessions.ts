import { createContext, useContext } from 'react'
import type { LaunchOptions, LaunchProgress, LogLevel } from '@/bindings'

const MAX_LOG_LINES = 10_000

export type SessionStatus = 'idle' | 'preparing' | 'running'

export interface LogLine {
  line: string
  level: LogLevel
}

export interface Session {
  status: SessionStatus
  progress: LaunchProgress | null
  error: string | null
  exitCode: number | null
  log: LogLine[]
}

const IDLE: Session = { status: 'idle', progress: null, error: null, exitCode: null, log: [] }

type Action =
  | { type: 'preparing'; id: string }
  | { type: 'progress'; id: string; progress: LaunchProgress }
  | { type: 'running'; id: string }
  | { type: 'failed'; id: string; error: string }
  | { type: 'log'; id: string; line: LogLine }
  | { type: 'exited'; id: string; exitCode: number | null }
  | { type: 'clearLog'; id: string }

export function reducer(sessions: Record<string, Session>, action: Action): Record<string, Session> {
  const current = sessions[action.id] ?? IDLE
  const update = (patch: Partial<Session>) => ({ ...sessions, [action.id]: { ...current, ...patch } })
  switch (action.type) {
    case 'preparing':
      return update({ status: 'preparing', progress: null, error: null, exitCode: null, log: [] })
    case 'progress':
      return update({ progress: action.progress })
    case 'running':
      return update({ status: 'running', progress: null })
    case 'failed':
      return update({ status: 'idle', progress: null, error: action.error })
    case 'log':
      return update({ log: [...current.log.slice(-(MAX_LOG_LINES - 1)), action.line] })
    case 'exited':
      return update({ status: 'idle', exitCode: action.exitCode })
    case 'clearLog':
      return update({ log: [] })
  }
}

interface SessionsContextValue {
  sessions: Record<string, Session>
  launch: (id: string, options?: LaunchOptions) => Promise<void>
  kill: (id: string) => Promise<void>
  clearLog: (id: string) => void
}

export const SessionsContext = createContext<SessionsContextValue | null>(null)

function useSessionsContext() {
  const context = useContext(SessionsContext)
  if (!context) throw new Error('useSession must be used inside SessionsProvider')
  return context
}

export function useSession(id: string) {
  const { sessions, launch, kill, clearLog } = useSessionsContext()
  return {
    session: sessions[id] ?? IDLE,
    launch: (options?: LaunchOptions) => launch(id, options),
    kill: () => kill(id),
    clearLog: () => clearLog(id),
  }
}

/** Ids of instances that are preparing or running. */
export function useActiveInstanceIds() {
  const { sessions } = useSessionsContext()
  return Object.entries(sessions)
    .filter(([, session]) => session.status !== 'idle')
    .map(([id]) => id)
}

/** Starts any instance; for one known instance prefer [`useSession`]. */
export function useLaunch() {
  return useSessionsContext().launch
}
