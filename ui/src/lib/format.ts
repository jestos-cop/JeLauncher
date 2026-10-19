import type { LaunchProgress } from '@/bindings'

export function describeProgress(progress: LaunchProgress | null): string {
  if (!progress) return 'Preparing…'
  switch (progress.stage) {
    case 'preparing':
      return 'Preparing…'
    case 'downloading':
      return `Downloading files ${progress.completed}/${progress.total}`
    case 'installingLoader':
      return 'Installing mod loader…'
    case 'starting':
      return 'Starting game…'
  }
}

export function progressPercent(progress: LaunchProgress | null): number | null {
  if (progress?.stage !== 'downloading' || progress.total === 0) return null
  return Math.round((progress.completed / progress.total) * 100)
}

export function formatPlayTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}
