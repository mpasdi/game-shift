import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { launchGame, listRunningGames, updateGame, deleteGame } from '../api.ts'
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
  // 运行和启动期间只允许资料编辑，启动配置和移除都必须等待退出。
  it.each(['runningGameIds', 'launchingGameIds'] as const)('guards mutations while %s is active', async (state) => {
    const store = useGamesStore()
    store.games = [...games]
    store[state] = ['game-1']
    const payload = { ...games[0], name: '新名称', favorite: true }
    vi.mocked(updateGame).mockResolvedValue({ ...games[0], ...payload })
    await store.updateGame(payload)
    expect(updateGame).toHaveBeenCalledTimes(1)
    for (const change of [{ exePath: 'D:\\Other.exe' }, { workDir: 'D:\\Other' }, { args: '--new' }]) {
      await expect(store.updateGame({ ...payload, ...change })).rejects.toThrow('修改启动配置')
    }
    expect(updateGame).toHaveBeenCalledTimes(1)
    await expect(store.deleteGame('game-1')).rejects.toThrow('再移除')
    expect(deleteGame).not.toHaveBeenCalled()
    expect(store[state]).toEqual(['game-1'])
    store[state] = []
    vi.mocked(deleteGame).mockResolvedValue(undefined)
    await store.deleteGame('game-1')
    expect(deleteGame).toHaveBeenCalledWith('game-1')
  })
  beforeEach(() => {
    // 每个测试使用全新的 Pinia，避免测试之间共享状态
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(listRunningGames).mockResolvedValue({ gameIds: [], error: null })
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

  // 测试场景：冷却结束但进程仍在运行时阻止启动；确认退出后允许再次启动。
  it('blocks a running game after cooldown and allows relaunch after exit', async () => {
    vi.mocked(launchGame).mockResolvedValue(games[0])
    vi.mocked(listRunningGames).mockResolvedValue({ gameIds: ['game-1'], error: null })
    const store = useGamesStore()
    await store.launchGame('game-1')
    vi.advanceTimersByTime(3000)
    expect(store.runningGameIds).toEqual(['game-1'])
    expect(await store.launchGame('game-1')).toBeNull()
    expect(launchGame).toHaveBeenCalledTimes(1)
    vi.mocked(listRunningGames).mockResolvedValue({ gameIds: [], error: null })
    await store.refreshRunningGames()
    await store.launchGame('game-1')
    expect(launchGame).toHaveBeenCalledTimes(2)
  })

  // 测试场景：同步异常保留运行锁定，后续成功快照恢复；旧快照不能覆盖新状态。
  it('preserves state on sync failure and ignores stale snapshots', async () => {
    const store = useGamesStore()
    store.runningGameIds = ['game-1']
    vi.mocked(listRunningGames).mockRejectedValueOnce(new Error('offline'))
    await store.refreshRunningGames()
    expect(store.runningGameIds).toEqual(['game-1'])
    expect(store.runtimeError).not.toBeNull()
    let finish!: (value: { gameIds: string[]; error: null }) => void
    vi.mocked(listRunningGames).mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve
      })
    )
    const oldRequest = store.refreshRunningGames()
    await store.refreshRunningGames()
    expect(store.runningGameIds).toEqual([])
    expect(store.runtimeError).toBeNull()
    finish({ gameIds: ['game-1'], error: null })
    await oldRequest
    expect(store.runningGameIds).toEqual([])
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
