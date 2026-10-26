import type { GamePlaytime } from '../types/game'

export const PLAYTIME_SCOPE = '仅统计通过 Game Shift 启动且管理器运行期间的游戏时长，历史时长不补算。'

export function formatPlayDuration(durationMs?: number | null): string {
  if (durationMs == null || !Number.isFinite(durationMs) || durationMs < 1000) return '—'
  const seconds = Math.floor(durationMs / 1000)
  if (seconds < 60) return `${seconds} 秒`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes} 分钟`
  const hours = Math.floor(minutes / 60)
  const remainder = minutes % 60
  return remainder ? `${hours} 小时 ${remainder} 分钟` : `${hours} 小时`
}

export function playtimeDescription(playtime?: GamePlaytime): string {
  const lines = [PLAYTIME_SCOPE]
  const duration = formatPlayDuration(playtime?.totalDurationMs)
  if (duration === '—') return `${lines[0]}\n暂无时长记录。`
  lines.push(`累计 ${duration}`)
  return lines.join('\n')
}
