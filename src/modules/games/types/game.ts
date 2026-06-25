export type GameFilter = 'all' | 'favorite' | 'recent'

export interface Game {
  id: string
  name: string
  exePath: string
  folderPath: string
  icon?: string | null
  args?: string | null
  workDir?: string | null
  favorite: boolean
  playCount: number
  lastPlayTime?: number | null
  createTime: number
  updateTime: number
}

export interface CreateGamePayload {
  name: string
  exePath: string
  workDir?: string | null
  args?: string | null
}

export interface UpdateGamePayload extends CreateGamePayload {
  id: string
  favorite: boolean
}
