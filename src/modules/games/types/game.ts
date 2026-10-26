import type { CoverSelection } from './cover'

export type GameFilter = 'all' | 'favorite' | 'recent'

export interface Game {
  id: string
  name: string
  exePath: string
  folderPath: string
  icon?: string | null
  cover?: string | null
  args?: string | null
  workDir?: string | null
  favorite: boolean
  favoriteTime?: number | null
  playCount: number
  lastPlayTime?: number | null
  createTime: number
  updateTime: number
  playtime?: GamePlaytime
}

export interface GamePlaytime {
  gameId: string
  totalDurationMs: number | null
  lastSessionDurationMs: number | null
  lastSessionEndReason: 'game_exited' | 'manager_exited' | 'interrupted' | null
  currentSessionDurationMs: number | null
}

export interface CreateGamePayload {
  name: string
  exePath: string
  workDir?: string | null
  args?: string | null
  coverPath?: string | null
  coverSelection?: CoverSelection | null
}

export interface UpdateGamePayload extends CreateGamePayload {
  id: string
  favorite: boolean
}

export interface ScanCandidate {
  name: string
  exePath: string
  folderPath: string
  exeFileName: string
  exists: boolean
  recommended: boolean
  confidence: number
  reasons: string[]
}
