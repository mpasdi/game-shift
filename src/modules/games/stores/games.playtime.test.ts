import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useGamesStore } from './games'
import { listGames, listRunningGames } from '../api'
import type { RunningGamesSnapshot } from '../api'
import type { Game, GamePlaytime } from '../types/game'

vi.mock('../api')
const game: Game = {
  id: 'game',
  name: '测试游戏',
  exePath: 'C:\\Game.exe',
  folderPath: 'C:\\',
  favorite: true,
  playCount: 2,
  createTime: 1,
  updateTime: 1
}
const running: GamePlaytime = {
  gameId: game.id,
  totalDurationMs: 120_000,
  lastSessionDurationMs: 60_000,
  lastSessionEndReason: null,
  currentSessionDurationMs: 60_000
}
beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
})

describe('game playtime synchronization', () => {
  // 测试场景：进程退出后立即展示最终时长，加载游戏列表的先后顺序不覆盖新进度。
  it('decorates loaded games and keeps final playtime without reloading the library', async () => {
    const store = useGamesStore()
    vi.mocked(listRunningGames).mockResolvedValue({ gameIds: [game.id], error: null, playtimes: [running] })
    await store.refreshRunningGames()
    vi.mocked(listGames).mockResolvedValue([game])
    await store.loadGames()
    expect(store.filteredGames[0].playtime).toEqual(running)
    const finished = { ...running, currentSessionDurationMs: null, lastSessionEndReason: 'game_exited' as const }
    vi.mocked(listRunningGames).mockResolvedValue({ gameIds: [], error: null, playtimes: [finished] })
    await store.refreshRunningGames()
    expect(store.gamesWithPlaytime[0].playtime).toEqual(finished)
    expect(listGames).toHaveBeenCalledTimes(1)
    store.upsertGame({ ...game, name: '更新名称' })
    expect(store.filteredGames[0].playtime).toEqual(finished)
    expect(store.filteredGames[0].name).toBe('更新名称')
  })

  // 测试场景：断线保留最后确认的时长，过期快照不能覆盖新的最终记录。
  it('preserves confirmed time on failure and ignores late responses', async () => {
    const store = useGamesStore()
    store.games = [game]
    store.playtimes = { [game.id]: running }
    vi.mocked(listRunningGames).mockRejectedValueOnce(new Error('offline'))
    await store.refreshRunningGames()
    expect(store.gamesWithPlaytime[0].playtime).toEqual(running)
    let resolveOld!: (snapshot: RunningGamesSnapshot) => void
    vi.mocked(listRunningGames).mockReturnValueOnce(
      new Promise((resolve) => {
        resolveOld = resolve
      })
    )
    const pending = store.refreshRunningGames()
    const finished = { ...running, currentSessionDurationMs: null, lastSessionEndReason: 'interrupted' as const }
    vi.mocked(listRunningGames).mockResolvedValueOnce({ gameIds: [], error: null, playtimes: [finished] })
    await store.refreshRunningGames()
    resolveOld({ gameIds: [game.id], error: null, playtimes: [running] })
    await pending
    expect(store.gamesWithPlaytime[0].playtime).toEqual(finished)
  })
})
