import { invoke } from '@tauri-apps/api/core'

export interface AppInfo {
  name: string
  version: string
  identifier: string
  dataDir: string
  databasePath: string
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
