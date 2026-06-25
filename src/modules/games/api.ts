import { invoke } from '@tauri-apps/api/core'
import type { CreateGamePayload, Game } from './types/game'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function listGames() {
  if (!isTauriRuntime()) return []

  return invoke<Game[]>('list_games_command')
}

export async function createGame(payload: CreateGamePayload) {
  if (!isTauriRuntime()) {
    throw new Error('当前环境不支持写入本地游戏库，请在 Tauri 桌面应用中使用')
  }

  return invoke<Game>('create_game_command', { payload })
}
