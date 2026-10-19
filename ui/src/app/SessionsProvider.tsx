import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useReducer, type ReactNode } from 'react'
import { reducer, SessionsContext } from '@/app/sessions'
import { commands, events, type LaunchOptions } from '@/bindings'

const DEFAULT_LAUNCH: LaunchOptions = { quickPlay: null, demo: false }

export function SessionsProvider({ children }: { children: ReactNode }) {
  const [sessions, dispatch] = useReducer(reducer, {})
  const queryClient = useQueryClient()

  useEffect(() => {
    const subscriptions = [
      events.launchProgressEvent.listen(({ payload }) =>
        dispatch({ type: 'progress', id: payload.instanceId, progress: payload.progress }),
      ),
      events.instanceLogEvent.listen(({ payload }) =>
        dispatch({ type: 'log', id: payload.instanceId, line: { line: payload.line, level: payload.level } }),
      ),
      events.instanceExitedEvent.listen(({ payload }) => {
        dispatch({ type: 'exited', id: payload.instanceId, exitCode: payload.exitCode })
        void queryClient.invalidateQueries({ queryKey: ['instances'] })
      }),
    ]
    void commands.listRunningInstances().then((result) => {
      if (result.status === 'ok') result.data.forEach((id) => dispatch({ type: 'running', id }))
    })
    return () => subscriptions.forEach((subscription) => void subscription.then((unlisten) => unlisten()))
  }, [queryClient])

  const launch = useCallback(async (id: string, options: LaunchOptions = DEFAULT_LAUNCH) => {
    dispatch({ type: 'preparing', id })
    const result = await commands.launchInstance(id, options)
    if (result.status === 'ok') dispatch({ type: 'running', id })
    else dispatch({ type: 'failed', id, error: result.error.message })
  }, [])

  const kill = useCallback(async (id: string) => {
    const result = await commands.killInstance(id)
    if (result.status === 'error') dispatch({ type: 'failed', id, error: result.error.message })
  }, [])

  const clearLog = useCallback((id: string) => dispatch({ type: 'clearLog', id }), [])

  const value = useMemo(() => ({ sessions, launch, kill, clearLog }), [sessions, launch, kill, clearLog])
  return <SessionsContext value={value}>{children}</SessionsContext>
}
