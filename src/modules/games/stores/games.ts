import { defineStore } from 'pinia'
import { createGame, listGames } from '../api'
import type { CreateGamePayload, Game, GameFilter } from '../types/game'

interface GamesState {
  games: Game[]
  searchText: string
  activeFilter: GameFilter
  isLoading: boolean
  isSaving: boolean
  errorMessage: string | null
}

export const useGamesStore = defineStore('games', {
  state: (): GamesState => ({
    games: [],
    searchText: '',
    activeFilter: 'all',
    isLoading: false,
    isSaving: false,
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
          return game.name.toLowerCase().includes(keyword) || game.exePath.toLowerCase().includes(keyword)
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
