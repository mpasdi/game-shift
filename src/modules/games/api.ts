import { invoke } from '@tauri-apps/api/core'
import type { Game } from './types/game'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function listGames() {
  if (!isTauriRuntime()) return []

  return invoke<Game[]>('list_games_command')
}
