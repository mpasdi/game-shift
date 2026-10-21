import { defineStore } from 'pinia'
import { createGame, deleteGame, launchGame, listGames, listRunningGames, scanGames, updateGame } from '../api'
import type { CreateGamePayload, Game, GameFilter, ScanCandidate, UpdateGamePayload } from '../types/game'

const LAUNCH_COOLDOWN_MS = 3000

function sortGames(left: Game, right: Game, filter: GameFilter) {
  if (filter === 'recent') {
    return (right.lastPlayTime ?? 0) - (left.lastPlayTime ?? 0)
  }

  if (filter === 'favorite') {
    return (right.favoriteTime ?? right.updateTime) - (left.favoriteTime ?? left.updateTime)
  }

  return right.createTime - left.createTime
}

interface GamesState {
  games: Game[]
  searchText: string
  activeFilter: GameFilter
  isLoading: boolean
  isSaving: boolean
  isScanning: boolean
  launchingGameIds: string[]
  runningGameIds: string[]
  runtimeError: string | null
  runtimeRevision: number
  errorMessage: string | null
  libraryErrorMessage: string | null
}

export const useGamesStore = defineStore('games', {
  state: (): GamesState => ({
    games: [],
    searchText: '',
    activeFilter: 'all',
    isLoading: false,
    isSaving: false,
    isScanning: false,
    launchingGameIds: [],
    runningGameIds: [],
    runtimeError: null,
    runtimeRevision: 0,
    errorMessage: null,
    libraryErrorMessage: null
  }),
  getters: {
    filteredGames(state): Game[] {
      const keyword = state.searchText.trim().toLowerCase()
      return state.games
        .filter((game) => {
          if (state.activeFilter === 'favorite' && !game.favorite) return false
          if (state.activeFilter === 'recent' && !game.lastPlayTime) return false
          if (!keyword) return true
          const exeFileName = game.exePath.split(/[\\/]/).pop() ?? game.exePath
          return game.name.toLowerCase().includes(keyword) || exeFileName.toLowerCase().includes(keyword)
        })
        .sort((left, right) => sortGames(left, right, state.activeFilter))
    }
  },
  actions: {
    async refreshRunningGames() {
      const revision = ++this.runtimeRevision
      try {
        const snapshot = await listRunningGames()
        if (revision !== this.runtimeRevision) return
        this.runningGameIds = snapshot.gameIds
        this.runtimeError = snapshot.error
      } catch {
        if (revision === this.runtimeRevision) {
          this.runtimeError = '运行状态同步失败，暂时保留上次状态，稍后自动重试'
        }
      }
    },
    async loadGames() {
      this.isLoading = true
      this.libraryErrorMessage = null

      try {
        this.games = await listGames()
      } catch (error) {
        this.libraryErrorMessage = error instanceof Error ? error.message : String(error)
      } finally {
        this.isLoading = false
      }
    },
    async refreshGames() {
      await this.loadGames()
    },
    clearErrorMessage() {
      this.errorMessage = null
    },
    async createGame(payload: CreateGamePayload) {
      this.isSaving = true
      this.errorMessage = null

      try {
        const game = await createGame(payload)
        this.upsertGame(game)
        return game
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        this.isSaving = false
      }
    },
    async updateGame(payload: UpdateGamePayload) {
      this.isSaving = true
      this.errorMessage = null

      try {
        const current = this.games.find((game) => game.id === payload.id)
        if (
          current &&
          (this.runningGameIds.includes(payload.id) || this.launchingGameIds.includes(payload.id)) &&
          (payload.exePath.trim() !== current.exePath ||
            (payload.workDir?.trim() || current.folderPath) !== (current.workDir || current.folderPath) ||
            (payload.args?.trim() || '') !== (current.args || ''))
        )
          throw new Error('游戏正在运行，请先退出游戏再修改启动配置')
        const game = await updateGame(payload)
        this.upsertGame(game)
        return game
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        this.isSaving = false
      }
    },
    async deleteGame(id: string) {
      this.isSaving = true
      this.errorMessage = null

      try {
        if (this.runningGameIds.includes(id) || this.launchingGameIds.includes(id)) {
          throw new Error('游戏正在运行，请先退出游戏再移除')
        }
        await deleteGame(id)
        this.games = this.games.filter((game) => game.id !== id)
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        this.isSaving = false
      }
    },
    async launchGame(id: string) {
      if (this.launchingGameIds.includes(id) || this.runningGameIds.includes(id)) return null

      ++this.runtimeRevision
      this.launchingGameIds.push(id)
      this.errorMessage = null

      try {
        const game = await launchGame(id)
        if (!this.runningGameIds.includes(id)) this.runningGameIds.push(id)
        this.upsertGame(game)
        return game
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        await this.refreshRunningGames()
        window.setTimeout(() => {
          this.launchingGameIds = this.launchingGameIds.filter((gameId) => gameId !== id)
        }, LAUNCH_COOLDOWN_MS)
      }
    },
    async scanGames(directory: string): Promise<ScanCandidate[]> {
      this.isScanning = true
      this.errorMessage = null

      try {
        return await scanGames(directory)
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        this.isScanning = false
      }
    },
    upsertGame(game: Game) {
      const index = this.games.findIndex((item) => item.id === game.id)
      if (index >= 0) {
        this.games[index] = game
        return
      }

      this.games.push(game)
    },
    setFilter(filter: GameFilter) {
      this.activeFilter = filter
    },
    countByFilter(filter: GameFilter) {
      if (filter === 'favorite') return this.games.filter((game) => game.favorite).length
      if (filter === 'recent') return this.games.filter((game) => game.lastPlayTime).length
      return this.games.length
    }
  }
})
