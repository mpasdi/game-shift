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
