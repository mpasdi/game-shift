import { beforeEach, describe, vi, it, expect } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { listGames, createGame, updateGame, deleteGame } from '../api.ts'
import { useGamesStore } from './games.ts'

// type
import type { Game, CreateGamePayload, UpdateGamePayload } from '../types/game.ts'

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

describe('Game Stores', () => {
  beforeEach(() => {
    // 每个测试使用全新的 Pinia，避免测试之间共享状态
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  // 测试 loadGames
  // 游戏列表加载成功
  it('load game list successfully', async () => {
    vi.mocked(listGames).mockResolvedValue(games)

    const gamesStore = useGamesStore()

    // Act：执行我们要测试的行为
    await gamesStore.loadGames()
    expect(listGames).toHaveBeenCalledTimes(1)
    expect(gamesStore.games).toEqual(games)
    expect(gamesStore.isLoading).toBe(false)
    expect(gamesStore.libraryErrorMessage).toBeNull()
  })

  it('keeps loading status while loading games ', async () => {
    vi.mocked(listGames).mockResolvedValue(games)

    const gamesStore = useGamesStore()

    // Act：执行我们要测试的行为
    const loadGamesRequest = gamesStore.loadGames()
    expect(gamesStore.isLoading).toBe(true)

    await loadGamesRequest
    expect(listGames).toHaveBeenCalledTimes(1)
    expect(gamesStore.games).toEqual(games)
    expect(gamesStore.isLoading).toBe(false)
    expect(gamesStore.libraryErrorMessage).toBeNull()
  })

  it('records error when loading game fails', async () => {
    vi.mocked(listGames).mockRejectedValue(new Error('加载异常'))

    const gamesStore = useGamesStore()
    await gamesStore.loadGames()
    expect(listGames).toHaveBeenCalledTimes(1)
    expect(gamesStore.games).toEqual([])
    expect(gamesStore.libraryErrorMessage).toBe('加载异常')
    expect(gamesStore.isLoading).toBe(false)
  })

  // 测试 create
  it('create game successfully', async () => {
    vi.mocked(createGame).mockResolvedValue(games[0])

    const createGamePayload: CreateGamePayload = {
      name: '空洞骑士',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe'
    }
    const gamesStore = useGamesStore()
    const createdGame = await gamesStore.createGame(createGamePayload)
    expect(createGame).toHaveBeenCalledTimes(1)
    expect(createGame).toHaveBeenCalledWith(createGamePayload)
    expect(createdGame).toEqual(games[0])
    expect(gamesStore.games).toEqual(games)
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.errorMessage).toBeNull()
  })

  it('keeps saving status while creating game', async () => {
    vi.mocked(createGame).mockResolvedValue(games[0])

    const createGamePayload: CreateGamePayload = {
      name: '空洞骑士',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe'
    }
    const gamesStore = useGamesStore()
    const createdGame = gamesStore.createGame(createGamePayload)
    expect(gamesStore.isSaving).toBe(true)
    expect(gamesStore.games).toEqual([])

    await createdGame
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.games).toEqual(games)
  })

  it('records and rethrows error when creating game fails', async () => {
    vi.mocked(createGame).mockRejectedValue(new Error('创建游戏失败'))

    const createGamePayload: CreateGamePayload = {
      name: '空洞骑士',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe'
    }
    const gamesStore = useGamesStore()
    const createResult = gamesStore.createGame(createGamePayload)
    await expect(createResult).rejects.toThrow('创建游戏失败')

    expect(createGame).toHaveBeenCalledWith(createGamePayload)
    expect(gamesStore.errorMessage).toBe('创建游戏失败')
    expect(gamesStore.games).toEqual([])
    expect(gamesStore.isSaving).toBe(false)
  })

  // update game 测试
  it('update game successfully', async () => {
    const payload: UpdateGamePayload = {
      id: 'game-1',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
      name: '王者荣耀',
      favorite: true
    }

    const expectResult = {
      ...games[0],
      ...payload
    }

    vi.mocked(updateGame).mockResolvedValue(expectResult)

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]
    const updatedGame = await gamesStore.updateGame(payload)
    expect(updateGame).toHaveBeenCalledWith(payload)
    expect(updatedGame).toEqual(expectResult)
    expect(gamesStore.games).toEqual([expectResult])

    expect(gamesStore.games[0].name).toBe('王者荣耀')
    expect(gamesStore.games[0].favorite).toBe(true)

    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.errorMessage).toBeNull()
  })

  it('keeps saving status while updating game', async () => {
    const payload: UpdateGamePayload = {
      id: 'game-1',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
      name: '王者荣耀',
      favorite: true
    }

    const expectResult = {
      ...games[0],
      ...payload
    }

    vi.mocked(updateGame).mockResolvedValue(expectResult)

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]

    const updateGamePromise = gamesStore.updateGame(payload)
    expect(gamesStore.isSaving).toBe(true)
    expect(gamesStore.games[0].name).toBe('空洞骑士')
    expect(gamesStore.games[0].favorite).toBe(false)

    await updateGamePromise
    expect(gamesStore.games[0].name).toBe('王者荣耀')
    expect(gamesStore.games[0].favorite).toBe(true)
    expect(gamesStore.games).toEqual([expectResult])
    expect(gamesStore.isSaving).toBe(false)
  })

  it('records and rethrows error when update game fails', async () => {
    vi.mocked(updateGame).mockRejectedValue(new Error('更新失败'))

    const payload: UpdateGamePayload = {
      id: 'game-1',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
      name: '王者荣耀',
      favorite: true
    }

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]
    const result = gamesStore.updateGame(payload)
    await expect(result).rejects.toThrow('更新失败')

    expect(updateGame).toHaveBeenCalledTimes(1)
    expect(updateGame).toHaveBeenCalledWith(payload)
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.errorMessage).toBe('更新失败')
    expect(gamesStore.games).toEqual(games)
    expect(gamesStore.games[0].name).toBe('空洞骑士')
    expect(gamesStore.games[0].favorite).toBe(false)
  })

  it('deletes game successfully ', async () => {
    const anotherGame: Game = {
      ...games[0],
      id: 'game-2',
      name: '哈迪斯',
      exePath: 'D:\\Games\\Hades\\Hades.exe',
      folderPath: 'D:\\Games\\Hades'
    }

    const id = 'game-2'
    vi.mocked(deleteGame).mockResolvedValue()

    const gamesStore = useGamesStore()
    gamesStore.games = [...games, anotherGame]
    await gamesStore.deleteGame(id)

    expect(deleteGame).toHaveBeenCalledTimes(1)
    expect(deleteGame).toHaveBeenCalledWith(id)
    expect(gamesStore.games).toEqual([...games])
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.errorMessage).toBeNull()
  })

  it('keeps saving status while deleting game', async () => {
    const anotherGame: Game = {
      ...games[0],
      id: 'game-2',
      name: '哈迪斯',
      exePath: 'D:\\Games\\Hades\\Hades.exe',
      folderPath: 'D:\\Games\\Hades'
    }

    const id = 'game-2'
    vi.mocked(deleteGame).mockResolvedValue()

    const gamesStore = useGamesStore()
    gamesStore.games = [...games, anotherGame]
    const deleteGamePromise = gamesStore.deleteGame(id)
    expect(gamesStore.isSaving).toBe(true)
    expect(gamesStore.games).toEqual([...games, anotherGame])

    await deleteGamePromise
    expect(deleteGame).toHaveBeenCalledTimes(1)
    expect(deleteGame).toHaveBeenCalledWith(id)
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.games).toEqual([...games])
  })

  it('records and rethrows error when deleting game fails', async () => {
    const id = 'game-1'
    vi.mocked(deleteGame).mockRejectedValue(new Error('删除失败'))

    const gamesStore = useGamesStore()
    gamesStore.games = [...games]
    const deleteGamePromise = gamesStore.deleteGame(id)
    await expect(deleteGamePromise).rejects.toThrow('删除失败')

    expect(deleteGame).toHaveBeenCalledTimes(1)
    expect(deleteGame).toHaveBeenCalledWith(id)
    expect(gamesStore.games).toEqual(games)
    expect(gamesStore.isSaving).toBe(false)
    expect(gamesStore.errorMessage).toBe('删除失败')
  })

  //   test filteredGames
  it('filters games by name ignoring case and whitespace', () => {
    const otherData = {
      ...games[0],
      id: 'game-2',
      name: 'test game 1',
      favorite: true
    }
    const gamesStore = useGamesStore()
    gamesStore.games = [...games, otherData]
    gamesStore.searchText = 'Test '

    expect(gamesStore.filteredGames).toEqual([otherData])
  })

  it('filters games by executable file name  ', () => {
    const otherData = {
      ...games[0],
      id: 'game-2',
      name: 'test game 1',
      favorite: true,
      exePath: 'D:\\Games\\test path \\use to testgame.exe'
    }
    const gamesStore = useGamesStore()
    gamesStore.games = [...games, otherData]
    gamesStore.searchText = 'use'

    expect(gamesStore.filteredGames).toEqual([otherData])
  })

  // setFilter
  it('filters favorite game', () => {
    const otherData = {
      ...games[0],
      id: 'game-2',
      name: 'test game 1',
      favorite: true,
      exePath: 'D:\\Games\\test path \\use to testgame.exe'
    }
    const gamesStore = useGamesStore()
    gamesStore.games = [...games, otherData]
    gamesStore.setFilter('favorite')

    expect(gamesStore.activeFilter).toBe('favorite')
    expect(gamesStore.filteredGames).toEqual([otherData])
  })

  it('filters last play time game', () => {
    const otherData = {
      ...games[0],
      id: 'game-2',
      name: 'test game 1',
      favorite: true,
      exePath: 'D:\\Games\\test path \\use to testgame.exe',
      lastPlayTime: 100
    }
    const gamesStore = useGamesStore()
    gamesStore.games = [...games, otherData]
    gamesStore.setFilter('recent')

    expect(gamesStore.activeFilter).toBe('recent')
    expect(gamesStore.filteredGames).toEqual([otherData])
  })

  it('sort all game by createTime', () => {
    const otherData = {
      ...games[0],
      id: 'game-2',
      name: 'test game 1',
      favorite: true,
      exePath: 'D:\\Games\\test path \\use to testgame.exe',
      createTime: 131356121
    }
    const gamesStore = useGamesStore()
    gamesStore.games = [...games, otherData]
    gamesStore.setFilter('all')

    expect(gamesStore.activeFilter).toBe('all')
    expect(gamesStore.filteredGames.map((game) => game.id)).toEqual(['game-2', 'game-1'])
  })

  it('sort favorite game by favorite time', () => {
    const otherData: Game[] = [
      games[0],
      { ...games[0], favorite: true, favoriteTime: 100, id: 'game-2' },
      { ...games[0], favorite: true, favoriteTime: 200, id: 'game-3' }
    ]
    const gamesStore = useGamesStore()
    gamesStore.games = [...otherData]
    gamesStore.setFilter('favorite')

    expect(gamesStore.activeFilter).toBe('favorite')
    expect(gamesStore.filteredGames.map((game) => game.id)).toEqual(['game-3', 'game-2'])
  })

  it('sort lastPlayTime game by last play time', () => {
    const otherData: Game[] = [
      games[0],
      { ...games[0], lastPlayTime: 100, id: 'game-2' },
      { ...games[0], lastPlayTime: 200, id: 'game-3' }
    ]
    const gamesStore = useGamesStore()
    gamesStore.games = [...otherData]
    gamesStore.setFilter('recent')

    expect(gamesStore.activeFilter).toBe('recent')
    expect(gamesStore.filteredGames.map((game) => game.id)).toEqual(['game-3', 'game-2'])
  })

  it('counts games by filter', () => {
    const gamesStore = useGamesStore()

    gamesStore.games = [
      { ...games[0], id: 'game-1' },
      { ...games[0], id: 'game-2', favorite: true },
      { ...games[0], id: 'game-3', favorite: true, lastPlayTime: 1000 }
    ]

    expect(gamesStore.countByFilter('all')).toBe(3)
    expect(gamesStore.countByFilter('favorite')).toBe(2)
    expect(gamesStore.countByFilter('recent')).toBe(1)
  })
})
