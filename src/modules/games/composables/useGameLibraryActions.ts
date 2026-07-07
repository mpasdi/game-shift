import type { InjectionKey, Ref } from 'vue'
import { inject } from 'vue'
import type { Game } from '../types/game'

export type GameViewMode = 'grid' | 'list'

export interface GameLibraryActions {
  viewMode: Readonly<Ref<GameViewMode>>
  launchingGameIds: Ref<string[]>
  setViewMode: (viewMode: GameViewMode) => void
  openEditGameDialog: (game: Game) => void
  openRemoveGameDialog: (game: Game) => void
  launchGame: (game: Game) => Promise<void>
  toggleFavorite: (game: Game) => Promise<void>
}

export const gameLibraryActionsKey: InjectionKey<GameLibraryActions> = Symbol('gameLibraryActions')

export function useGameLibraryActions() {
  const actions = inject(gameLibraryActionsKey)

  if (!actions) {
    throw new Error('Game library actions are not provided.')
  }

  return actions
}
