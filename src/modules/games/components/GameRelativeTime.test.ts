import { afterEach, describe, expect, it, vi } from 'vitest'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import { h, KeepAlive, nextTick, ref } from 'vue'
import GameGridCard from './GameGridCard.vue'
import GameQuickCard from './GameQuickCard.vue'
import GameTable from './GameTable.vue'
import type { Game } from '../types/game'

enableAutoUnmount(afterEach)
afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
})

const game: Game = {
  id: 'recent-game',
  name: '测试游戏',
  exePath: 'C:\\Games\\Game.exe',
  folderPath: 'C:\\Games',
  favorite: false,
  playCount: 1,
  lastPlayTime: new Date(2026, 9, 25, 9, 5).getTime(),
  createTime: 1,
  updateTime: 1
}

function setDay(day: number) {
  vi.setSystemTime(new Date(2026, 9, day, 12))
}

const Presentations = {
  setup: () => () => [
    h(GameGridCard, { game }),
    h(GameQuickCard, { game, showLastPlayTime: true }),
    h(GameTable, { games: [game] })
  ]
}

describe('game relative time refresh', () => {
  // 不更新游戏数据，仅通过恢复窗口事件使正文与悬停日期同步跨日。
  it.each(['focus', 'pageshow'])('refreshes every presentation on window %s', async (event) => {
    vi.useFakeTimers()
    setDay(25)
    const wrapper = mount(Presentations)
    await nextTick()
    const labels = () => [
      wrapper.getComponent(GameGridCard).get('p'),
      wrapper.getComponent(GameQuickCard).get('p'),
      wrapper.getComponent(GameTable).get('.game-table__stat')
    ]
    expect(labels().map((label) => label.text())).toEqual(['今天 09:05 · 1 次', '今天 09:05', '今天 09:05'])

    setDay(26)
    window.dispatchEvent(new Event(event))
    await nextTick()
    expect(labels().map((label) => label.text())).toEqual(['昨天 09:05 · 1 次', '昨天 09:05', '昨天 09:05'])
    for (const label of labels()) expect(label.attributes('title')).toBe(label.text())

    setDay(27)
    window.dispatchEvent(new Event(event))
    await nextTick()
    expect(labels().map((label) => label.text())).toEqual([
      '2026/10/25 09:05 · 1 次',
      '2026/10/25 09:05',
      '2026/10/25 09:05'
    ])
  })

  it('refreshes when the document becomes visible, without a recurring timer', async () => {
    vi.useFakeTimers()
    setDay(25)
    const wrapper = mount(Presentations)
    await nextTick()
    const visibility = vi.spyOn(document, 'visibilityState', 'get').mockReturnValue('hidden')

    setDay(26)
    document.dispatchEvent(new Event('visibilitychange'))
    await nextTick()
    expect(wrapper.getComponent(GameGridCard).text()).toContain('今天 09:05')

    visibility.mockReturnValue('visible')
    document.dispatchEvent(new Event('visibilitychange'))
    await nextTick()
    expect(wrapper.getComponent(GameGridCard).text()).toContain('昨天 09:05')
    expect(wrapper.getComponent(GameQuickCard).text()).toContain('昨天 09:05')
    expect(wrapper.getComponent(GameTable).text()).toContain('昨天 09:05')
    expect(vi.getTimerCount()).toBe(0)
  })

  // 同一共享作用域仍存活时，新页面挂载也必须重新读取当前日期。
  it('refreshes on re-entry while another card stays mounted', async () => {
    vi.useFakeTimers()
    setDay(25)
    const grid = mount(GameGridCard, { props: { game } })
    await nextTick()
    setDay(26)
    const quick = mount(GameQuickCard, { props: { game, showLastPlayTime: true } })
    await nextTick()
    expect(grid.text()).toContain('昨天 09:05')
    expect(quick.text()).toContain('昨天 09:05')

    quick.unmount()
    setDay(27)
    window.dispatchEvent(new Event('focus'))
    await nextTick()
    expect(grid.text()).toContain('2026/10/25 09:05')

    grid.unmount()
    setDay(26)
    const table = mount(GameTable, { props: { games: [game] } })
    await nextTick()
    expect(table.text()).toContain('昨天 09:05')
  })

  it('refreshes when a cached page is activated again', async () => {
    vi.useFakeTimers()
    setDay(25)
    const showingGames = ref(true)
    const OtherPage = { render: () => h('div', '其他页面') }
    const wrapper = mount({
      setup: () => () => h(KeepAlive, null, { default: () => h(showingGames.value ? Presentations : OtherPage) })
    })
    await nextTick()
    showingGames.value = false
    await nextTick()
    setDay(26)
    showingGames.value = true
    await nextTick()
    expect(wrapper.getComponent(GameGridCard).text()).toContain('昨天 09:05')
    expect(wrapper.getComponent(GameQuickCard).text()).toContain('昨天 09:05')
    expect(wrapper.getComponent(GameTable).text()).toContain('昨天 09:05')
  })

  it('uses the current date when game data changes before any resume event', async () => {
    vi.useFakeTimers()
    setDay(25)
    const grid = mount(GameGridCard, { props: { game } })
    const quick = mount(GameQuickCard, { props: { game, showLastPlayTime: true } })
    const table = mount(GameTable, { props: { games: [game] } })
    await nextTick()
    setDay(26)
    const updatedGame = { ...game, lastPlayTime: new Date(2026, 9, 26, 9, 5).getTime() }
    await grid.setProps({ game: updatedGame })
    await quick.setProps({ game: updatedGame })
    await table.setProps({ games: [updatedGame] })
    for (const wrapper of [grid, quick, table]) expect(wrapper.text()).toContain('今天 09:05')
  })

  it('removes resume listeners after the last presentation unmounts', async () => {
    vi.useFakeTimers()
    setDay(25)
    const windowAdd = vi.spyOn(window, 'addEventListener')
    const windowRemove = vi.spyOn(window, 'removeEventListener')
    const documentAdd = vi.spyOn(document, 'addEventListener')
    const documentRemove = vi.spyOn(document, 'removeEventListener')
    const wrapper = mount(Presentations)
    await nextTick()
    const windowListeners = windowAdd.mock.calls.filter(([event]) => ['focus', 'pageshow'].includes(event))
    const documentListeners = documentAdd.mock.calls.filter(([event]) => event === 'visibilitychange')
    expect(windowListeners).toHaveLength(2)
    expect(documentListeners).toHaveLength(1)
    wrapper.unmount()
    for (const [event, listener] of windowListeners) {
      expect(
        windowRemove.mock.calls.some(
          ([removedEvent, removedListener]) => removedEvent === event && removedListener === listener
        )
      ).toBe(true)
    }
    for (const [event, listener] of documentListeners) {
      expect(
        documentRemove.mock.calls.some(
          ([removedEvent, removedListener]) => removedEvent === event && removedListener === listener
        )
      ).toBe(true)
    }
    expect(vi.getTimerCount()).toBe(0)
  })
})
