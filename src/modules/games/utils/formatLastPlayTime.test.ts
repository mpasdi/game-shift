import { describe, expect, it } from 'vitest'
import { formatLastPlayTime } from './formatLastPlayTime'

describe('formatLastPlayTime', () => {
  it.each([undefined, null, 0])('returns the placeholder for timestamp %s', (timestamp) => {
    expect(formatLastPlayTime(timestamp)).toBe('无启动记录')
  })

  it('formats a time from today', () => {
    const now = new Date(2026, 9, 18, 12, 0).getTime()
    const playedAt = new Date(2026, 9, 18, 9, 5).getTime()

    expect(formatLastPlayTime(playedAt, now)).toBe(`今天 09:05`)
  })

  it('formats a time from yesterday', () => {
    const now = new Date(2026, 9, 18, 12, 0).getTime()
    const playedAt = new Date(2026, 9, 17, 9, 5).getTime()

    expect(formatLastPlayTime(playedAt, now)).toBe(`昨天 09:05`)
  })

  it('formats yesterday across a month boundary', () => {
    const now = new Date(2026, 9, 1, 0, 10).getTime()
    const playedAt = new Date(2026, 8, 30, 23, 50).getTime()

    expect(formatLastPlayTime(playedAt, now)).toBe('昨天 23:50')
  })

  it("formats yesterday across a year boundary'", () => {
    const now = new Date(2026, 0, 1, 0, 10).getTime()
    const playedAt = new Date(2025, 11, 31, 23, 10).getTime()

    expect(formatLastPlayTime(playedAt, now)).toBe('昨天 23:10')
  })

  it("formats a full date for records older than yesterday''", () => {
    const now = new Date(2026, 0, 3, 2, 10).getTime()
    const playedAt = new Date(2026, 0, 1, 2, 10).getTime()

    expect(formatLastPlayTime(playedAt, now)).toBe('2026/01/01 02:10')
  })
})
