import { describe, expect, it } from 'vitest'
import { formatPlayDuration, playtimeDescription } from './formatPlayDuration'
import type { GamePlaytime } from '../types/game'

describe('play duration display', () => {
  // 测试场景：缺失和零时长统一占位，非零时长按秒 / 分钟 / 小时向下取整。
  it.each([
    [undefined, '—'],
    [null, '—'],
    [-1, '—'],
    [NaN, '—'],
    [Infinity, '—'],
    [0, '—'],
    [999, '—'],
    [1000, '1 秒'],
    [32_000, '32 秒'],
    [59_999, '59 秒'],
    [60_000, '1 分钟'],
    [3_599_999, '59 分钟'],
    [3_600_000, '1 小时'],
    [3_660_000, '1 小时 1 分钟']
  ] as const)('formats %s as %s', (duration, expected) => {
    expect(formatPlayDuration(duration)).toBe(expected)
  })

  it('describes total time without exposing individual sessions', () => {
    const playtime: GamePlaytime = {
      gameId: 'game',
      totalDurationMs: 120_000,
      lastSessionDurationMs: 60_000,
      lastSessionEndReason: null,
      currentSessionDurationMs: 60_000
    }
    const description = playtimeDescription(playtime)
    expect(description).toContain('累计 2 分钟')
    expect(description).toContain('历史时长不补算')
    expect(description).not.toContain('本次')
    playtime.currentSessionDurationMs = null
    playtime.lastSessionEndReason = 'interrupted'
    expect(playtimeDescription(playtime)).toBe(description)
    playtime.lastSessionEndReason = 'manager_exited'
    expect(playtimeDescription(playtime)).toBe(description)
    expect(playtimeDescription()).toContain('暂无时长记录')
    expect(playtimeDescription({ ...playtime, totalDurationMs: 0 })).toContain('暂无时长记录')
  })
})
