import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { useRunningGames } from './useRunningGames'
import { useGamesStore } from '../stores/games'
import { listRunningGames } from '../api'

vi.mock('../api', () => ({ listRunningGames: vi.fn() }))
beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
  vi.useFakeTimers()
  vi.stubGlobal('__TAURI_INTERNALS__', {})
})
afterEach(() => {
  vi.clearAllTimers()
  vi.useRealTimers()
  vi.unstubAllGlobals()
})

const Host = {
  setup() {
    useRunningGames()
    return () => null
  }
}

describe('running game synchronization lifecycle', () => {
  // 测试场景：挂载读取当前会话，后续同步退出，卸载停止定时任务。
  it('refreshes running state and stops after unmount', async () => {
    vi.mocked(listRunningGames)
      .mockResolvedValueOnce({ gameIds: ['game'], error: null })
      .mockResolvedValue({ gameIds: [], error: null })
    const wrapper = mount(Host)
    try {
      await flushPromises()
      expect(useGamesStore().runningGameIds).toEqual(['game'])
      await vi.advanceTimersByTimeAsync(1000)
      expect(useGamesStore().runningGameIds).toEqual([])
    } finally {
      wrapper.unmount()
    }
    await vi.advanceTimersByTimeAsync(2000)
    expect(listRunningGames).toHaveBeenCalledTimes(2)
  })

  // 测试场景：慢请求期间不堆积轮询；卸载后的响应不得更新 Store 或重新启动定时任务。
  it('does not overlap requests or restart polling after a late response', async () => {
    let finish!: (value: { gameIds: string[]; error: null }) => void
    vi.mocked(listRunningGames).mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve
      })
    )
    const wrapper = mount(Host)
    await vi.advanceTimersByTimeAsync(5000)
    expect(listRunningGames).toHaveBeenCalledTimes(1)
    wrapper.unmount()
    finish({ gameIds: ['stale'], error: null })
    await flushPromises()
    expect(useGamesStore().runningGameIds).toEqual([])
    expect(vi.getTimerCount()).toBe(0)
  })
})
