import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { launchGame } from '../api.ts'
import { createPinia, setActivePinia } from 'pinia'
import { useGamesStore } from './games.ts'
// type
import type { Game } from '../types/game.ts'

// 测试时不调用真实的 Tauri 后端
vi.mock('../api')

const games: Game[] = [
  {
    id: 'game-1',
    name: '空洞骑士',
    exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
    folderPath: 'D:\\Games\\Hollow Knight',
    favorite: false,
    playCount: 3,
    createTime: 131356,
    updateTime: 2000
  }
]

describe('games store launch', () => {
  beforeEach(() => {
    // 每个测试使用全新的 Pinia，避免测试之间共享状态
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  // launchGame
  it('launches game successfully', async () => {
    const launchedGame: Game = {
      ...games[0],
      playCount: 4,
      lastPlayTime: 3000,
      updateTime: 3000
    }
    vi.mocked(launchGame).mockResolvedValue(launchedGame)

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]

    const game = await gamesStore.launchGame('game-1')

    expect(launchGame).toHaveBeenCalledTimes(1)
    expect(launchGame).toHaveBeenCalledWith('game-1')
    expect(game).toEqual(launchedGame)

    // 验收 gamesStore 中的状态是否重写
    expect(gamesStore.games).toEqual([launchedGame])
    expect(gamesStore.games[0].playCount).toBe(4)
    expect(gamesStore.games[0].lastPlayTime).toBe(3000)

    expect(gamesStore.errorMessage).toBeNull()
    expect(gamesStore.launchingGameIds).toEqual(['game-1'])

    vi.advanceTimersByTime(3000)
    expect(gamesStore.launchingGameIds).toEqual([])
  })

  // 重复启动拦截
  it('prevents duplicate launches during cooldown', async () => {
    const launchedGame: Game = {
      ...games[0],
      playCount: 4,
      lastPlayTime: 3000,
      updateTime: 3000
    }
    vi.mocked(launchGame).mockResolvedValue(launchedGame)

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]
    await gamesStore.launchGame('game-1')

    expect(launchGame).toHaveBeenCalledTimes(1)
    expect(gamesStore.launchingGameIds).toEqual(['game-1'])

    vi.advanceTimersByTime(2999)
    const duplicateResult = await gamesStore.launchGame('game-1')
    expect(duplicateResult).toBeNull()
    expect(launchGame).toHaveBeenCalledTimes(1)
    expect(gamesStore.launchingGameIds).toEqual(['game-1'])

    vi.advanceTimersByTime(1)
    expect(gamesStore.launchingGameIds).toEqual([])
  })

  it('records and rethrows error when launching game fails', async () => {
    vi.mocked(launchGame).mockRejectedValue(new Error('启动失败'))

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]
    const result = gamesStore.launchGame('game-1')

    await expect(result).rejects.toThrow('启动失败')
    expect(launchGame).toHaveBeenCalledTimes(1)
    expect(launchGame).toHaveBeenCalledWith('game-1')
    expect(gamesStore.errorMessage).toBe('启动失败')

    expect(gamesStore.games[0].playCount).toBe(3)

    expect(gamesStore.launchingGameIds).toEqual(['game-1'])
    vi.advanceTimersByTime(2999)
    expect(gamesStore.launchingGameIds).toEqual(['game-1'])
    vi.advanceTimersByTime(1)
    expect(gamesStore.launchingGameIds).toEqual([])
  })
})
