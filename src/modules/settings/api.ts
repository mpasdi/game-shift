import { invoke } from '@tauri-apps/api/core'

export interface AppInfo {
  name: string
  version: string
  identifier: string
  dataDir: string
  databasePath: string
}

export type OnlineCoverConfigState = 'disabled' | 'missingApiKey' | 'ready' | 'invalidApiKey'

export interface OnlineCoverSettings {
  enabled: boolean
  hasApiKey: boolean
  apiKeyHint?: string | null
  state: OnlineCoverConfigState
}

const browserOnlineCoverSettings: OnlineCoverSettings = {
  enabled: false,
  hasApiKey: false,
  apiKeyHint: null,
  state: 'disabled'
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function getAppInfo() {
  if (!isTauriRuntime()) {
    return {
      name: 'Game Shift',
      version: '0.1.0',
      identifier: 'com.gameshift.desktop',
      dataDir: '仅桌面应用可读取',
      databasePath: '仅桌面应用可读取'
    } satisfies AppInfo
  }

  return invoke<AppInfo>('app_info')
}

export async function getOnlineCoverSettings() {
  if (!isTauriRuntime()) return browserOnlineCoverSettings

  return invoke<OnlineCoverSettings>('get_online_cover_settings_command')
}

export async function setOnlineCoversEnabled(enabled: boolean) {
  assertTauriRuntime()
  return invoke<OnlineCoverSettings>('set_online_covers_enabled_command', { enabled })
}

export async function saveSteamGridDbApiKey(apiKey: string) {
  assertTauriRuntime()
  return invoke<OnlineCoverSettings>('save_steamgriddb_api_key_command', { apiKey })
}

export async function deleteSteamGridDbApiKey() {
  assertTauriRuntime()
  return invoke<OnlineCoverSettings>('delete_steamgriddb_api_key_command')
}

export async function testSteamGridDbConnection() {
  assertTauriRuntime()
  return invoke<OnlineCoverSettings>('test_steamgriddb_connection_command')
}

function assertTauriRuntime() {
  if (!isTauriRuntime()) {
    throw new Error('联网封面设置只能在 Tauri 桌面应用中修改')
  }
}
