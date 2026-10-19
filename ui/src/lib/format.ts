import type { LaunchProgress } from '@/bindings'
import { i18n } from '@/i18n'

export function describeProgress(progress: LaunchProgress | null): string {
  if (!progress) return i18n.t('launch:progress.preparing')
  switch (progress.stage) {
    case 'preparing':
      return i18n.t('launch:progress.preparing')
    case 'downloading':
      return i18n.t('launch:progress.downloading', { completed: progress.completed, total: progress.total })
    case 'installingLoader':
      return i18n.t('launch:progress.installingLoader')
    case 'starting':
      return i18n.t('launch:progress.starting')
  }
}

export function progressPercent(progress: LaunchProgress | null): number | null {
  if (progress?.stage !== 'downloading' || progress.total === 0) return null
  return Math.round((progress.completed / progress.total) * 100)
}

export function formatPlayTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) return i18n.t('main:playTime.hoursMinutes', { hours, minutes })
  return i18n.t('main:playTime.minutes', { minutes })
}
