import { defineStore } from 'pinia'
import { createGame, deleteGame, launchGame, listGames, updateGame } from '../api'
import type { CreateGamePayload, Game, GameFilter, UpdateGamePayload } from '../types/game'

const LAUNCH_COOLDOWN_MS = 3000

interface GamesState {
  games: Game[]
  searchText: string
  activeFilter: GameFilter
  isLoading: boolean
  isSaving: boolean
  launchingGameIds: string[]
  errorMessage: string | null
}

export const useGamesStore = defineStore('games', {
  state: (): GamesState => ({
    games: [],
    searchText: '',
    activeFilter: 'all',
    isLoading: false,
    isSaving: false,
    launchingGameIds: [],
    errorMessage: null
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
        .sort((left, right) => {
          if (Number(right.favorite) !== Number(left.favorite)) {
            return Number(right.favorite) - Number(left.favorite)
          }
          return (right.lastPlayTime ?? right.createTime) - (left.lastPlayTime ?? left.createTime)
        })
    }
  },
  actions: {
    async loadGames() {
      this.isLoading = true
      this.errorMessage = null

      try {
        this.games = await listGames()
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
      } finally {
        this.isLoading = false
      }
    },
    async refreshGames() {
      await this.loadGames()
    },
    async createGame(payload: CreateGamePayload) {
      this.isSaving = true
      this.errorMessage = null

      try {
        const game = await createGame(payload)
        await this.loadGames()
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
        const game = await updateGame(payload)
        await this.loadGames()
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
        await deleteGame(id)
        await this.loadGames()
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        this.isSaving = false
      }
    },
    async launchGame(id: string) {
      if (this.launchingGameIds.includes(id)) return null

      this.launchingGameIds.push(id)
      this.errorMessage = null

      try {
        const game = await launchGame(id)
        await this.loadGames()
        return game
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error)
        throw error
      } finally {
        window.setTimeout(() => {
          this.launchingGameIds = this.launchingGameIds.filter((gameId) => gameId !== id)
        }, LAUNCH_COOLDOWN_MS)
      }
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
