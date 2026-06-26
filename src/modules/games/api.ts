import { invoke } from '@tauri-apps/api/core'
import type { CreateGamePayload, Game, UpdateGamePayload } from './types/game'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function listGames() {
  if (!isTauriRuntime()) return []

  return invoke<Game[]>('list_games_command')
}

export async function getGame(id: string) {
  if (!isTauriRuntime()) return null

  return invoke<Game | null>('get_game_command', { id })
}

export async function createGame(payload: CreateGamePayload) {
  if (!isTauriRuntime()) {
    throw new Error('当前环境不支持写入本地游戏库，请在 Tauri 桌面应用中使用')
  }

  return invoke<Game>('create_game_command', { payload })
}

export async function updateGame(payload: UpdateGamePayload) {
  if (!isTauriRuntime()) {
    throw new Error('当前环境不支持更新本地游戏库，请在 Tauri 桌面应用中使用')
  }

  return invoke<Game>('update_game_command', { payload })
}

export async function deleteGame(id: string) {
  if (!isTauriRuntime()) {
    throw new Error('当前环境不支持移除本地游戏库记录，请在 Tauri 桌面应用中使用')
  }

  return invoke<void>('delete_game_command', { id })
}
