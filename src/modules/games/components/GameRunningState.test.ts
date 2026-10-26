import { afterEach, describe, expect, it } from 'vitest'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import GameGridCard from './GameGridCard.vue'
import GameQuickCard from './GameQuickCard.vue'
import GameTable from './GameTable.vue'
import type { Game } from '../types/game'

enableAutoUnmount(afterEach)
const game: Game = {
  id: 'running-game',
  name: '测试游戏',
  exePath: 'C:\\Games\\Game.exe',
  folderPath: 'C:\\Games',
  favorite: false,
  playCount: 1,
  createTime: 1,
  updateTime: 1
}

describe('game running state', () => {
  // 测试场景：列表展示累计时长，卡片保留日期；运行中、退出后及悬停均不展示单次时长。
  it('keeps cumulative playtime in tables and original metadata in cards', async () => {
    const timedGame: Game = {
      ...game,
      playtime: {
        gameId: game.id,
        totalDurationMs: 3_660_000,
        lastSessionDurationMs: 60_000,
        lastSessionEndReason: null,
        currentSessionDurationMs: 60_000
      }
    }
    const table = mount(GameTable, { props: { games: [timedGame], runningGameIds: [game.id] } })
    const grid = mount(GameGridCard, { props: { game: timedGame, isRunning: true } })
    const quick = mount(GameQuickCard, { props: { game: timedGame, isRunning: true, showLastPlayTime: true } })
    expect(table.text()).toContain('累计时长')
    expect(table.text()).toContain('1 小时 1 分钟')
    expect(grid.get('p').text()).toBe('无启动记录 · 1 次')
    expect(quick.get('p').text()).toBe('无启动记录')
    for (const wrapper of [table, grid, quick]) {
      expect(wrapper.html()).not.toContain('本次')
      expect(wrapper.html()).not.toContain('上次')
    }
    const ended: Game = {
      ...timedGame,
      playtime: { ...timedGame.playtime!, currentSessionDurationMs: null, lastSessionEndReason: 'interrupted' }
    }
    await table.setProps({ games: [ended], runningGameIds: [] })
    await grid.setProps({ game: ended, isRunning: false })
    await quick.setProps({ game: ended, isRunning: false })
    expect(table.text()).toContain('1 小时 1 分钟')
    expect(grid.get('p').text()).toBe('无启动记录 · 1 次')
    for (const card of [grid, quick]) {
      expect(card.html()).not.toContain('累计')
      expect(card.html()).not.toContain('1 小时 1 分钟')
    }
    for (const wrapper of [table, grid, quick]) {
      expect(wrapper.html()).not.toContain('本次')
      expect(wrapper.html()).not.toContain('上次')
      expect(wrapper.html()).not.toContain('中断')
    }
    await table.setProps({ games: [game] })
    expect(table.text()).toContain('—')
  })

  // 测试场景：两种卡片运行时禁用启动，退出后恢复正常操作。
  for (const [name, component] of [
    ['grid', GameGridCard],
    ['quick', GameQuickCard]
  ] as const) {
    it(`locks and restores the ${name} card launch button`, async () => {
      const wrapper = mount(component, { props: { game, isRunning: true } })
      expect(wrapper.find('.game-running-label').exists()).toBe(false)
      const button = wrapper.get<HTMLButtonElement>('[aria-label="游戏运行中"]')
      expect(button.element.disabled).toBe(true)
      expect(button.classes()).toContain('game-launch--running')
      expect(button.find('.game-running-dot').exists()).toBe(true)
      button.element.click()
      expect(wrapper.emitted('launch')).toBeUndefined()
      await wrapper.setProps({ isRunning: false })
      expect(wrapper.text()).not.toContain('运行中')
      wrapper.get<HTMLButtonElement>('[aria-label="启动游戏"]').element.click()
      expect(wrapper.emitted('launch')).toEqual([[game]])
    })
  }

  // 测试场景：列表的文字和图标启动入口均反映运行状态。
  it.each(['button', 'icon'] as const)('locks and restores the table %s launch button', async (launchStyle) => {
    const wrapper = mount(GameTable, {
      props: { games: [game], runningGameIds: [game.id], launchStyle, actionMode: 'launch-only' }
    })
    expect(wrapper.find('.game-running-label').exists()).toBe(false)
    const button = wrapper.get<HTMLButtonElement>('button')
    expect(button.element.disabled).toBe(true)
    expect(button.classes()).toContain('game-launch--running')
    expect(button.find('.game-running-dot').exists()).toBe(true)
    button.element.click()
    expect(wrapper.emitted('launch')).toBeUndefined()
    await wrapper.setProps({ runningGameIds: [] })
    expect(button.element.disabled).toBe(false)
    button.element.click()
    expect(wrapper.emitted('launch')).toEqual([[game]])
  })
})
