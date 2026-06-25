import { defineStore } from 'pinia'
import type { Game, GameFilter } from '../types/game'

interface GamesState {
  games: Game[]
  searchText: string
  activeFilter: GameFilter
}

export const useGamesStore = defineStore('games', {
  state: (): GamesState => ({
    games: [],
    searchText: '',
    activeFilter: 'all'
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
