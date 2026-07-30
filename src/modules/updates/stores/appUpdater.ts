import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { openUrl } from '@tauri-apps/plugin-opener'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { getErrorMessage, useToast } from '../../../shared/composables/useToast'

export type UpdatePhase = 'idle' | 'checking' | 'available' | 'downloading' | 'installing' | 'restarting' | 'failed'
export type UpdateCheckResult = 'never' | 'upToDate' | 'available' | 'failed'

export interface AvailableUpdateInfo {
  currentVersion: string
  version: string
  date?: string
  notes?: string
}

const RELEASES_URL = 'https://github.com/mpasdi/game-shift/releases'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function getUpdateCheckErrorMessage(error: unknown) {
  const message = getErrorMessage(error)
  if (/valid release JSON|failed to fetch|error sending request/i.test(message)) {
    return '暂时无法获取更新信息，请检查网络后重试。'
  }
  return message
}

export const useAppUpdaterStore = defineStore('appUpdater', () => {
  const toast = useToast()
  const phase = ref<UpdatePhase>('idle')
  const lastCheckResult = ref<UpdateCheckResult>('never')
  const lastError = ref<string | null>(null)
  const availableUpdate = ref<AvailableUpdateInfo | null>(null)
  const isDialogOpen = ref(false)
  const downloadedBytes = ref(0)
  const totalBytes = ref<number | null>(null)

  let pendingUpdate: Update | null = null

  const isChecking = computed(() => phase.value === 'checking')
  const isInstalling = computed(() => ['downloading', 'installing', 'restarting'].includes(phase.value))
  const downloadProgress = computed(() => {
    if (!totalBytes.value || totalBytes.value <= 0) return null
    return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100))
  })

  async function checkForUpdates() {
    if (isChecking.value || isInstalling.value) return

    phase.value = 'checking'
    lastError.value = null

    try {
      if (!isTauriRuntime()) {
        throw new Error('应用更新只能在已安装的 Game Shift 桌面应用中检查')
      }

      await releasePendingUpdate()
      availableUpdate.value = null
      isDialogOpen.value = false

      const update = await check({ timeout: 15_000 })
      if (!update) {
        lastCheckResult.value = 'upToDate'
        phase.value = 'idle'
        toast.success({ title: '当前已是最新版本' })
        return
      }

      pendingUpdate = update
      availableUpdate.value = {
        currentVersion: update.currentVersion,
        version: update.version,
        date: update.date,
        notes: update.body
      }
      lastCheckResult.value = 'available'
      phase.value = 'available'
      isDialogOpen.value = true
    } catch (error) {
      const message = getUpdateCheckErrorMessage(error)
      lastError.value = message
      lastCheckResult.value = 'failed'
      phase.value = 'idle'
      toast.error({ title: '检查更新失败', description: message })
    }
  }

  async function installUpdate() {
    if (!pendingUpdate || isInstalling.value) return

    lastError.value = null
    downloadedBytes.value = 0
    totalBytes.value = null
    phase.value = 'downloading'

    try {
      await pendingUpdate.download((event) => {
        if (event.event === 'Started') {
          totalBytes.value = event.data.contentLength ?? null
          return
        }

        if (event.event === 'Progress') {
          downloadedBytes.value += event.data.chunkLength
        }
      })

      phase.value = 'installing'
      await pendingUpdate.install()
      phase.value = 'restarting'
      await relaunch()
    } catch (error) {
      lastError.value = getErrorMessage(error)
      phase.value = 'failed'
    }
  }

  function dismissUpdate() {
    if (isInstalling.value) return
    isDialogOpen.value = false
  }

  function openUpdateDialog() {
    if (!pendingUpdate || !availableUpdate.value) return
    isDialogOpen.value = true
  }

  async function openReleasesPage() {
    try {
      await openUrl(RELEASES_URL)
    } catch (error) {
      toast.error({ title: '无法打开下载页面', description: getErrorMessage(error) })
    }
  }

  async function releasePendingUpdate() {
    if (!pendingUpdate) return

    try {
      await pendingUpdate.close()
    } catch {
      // 资源释放失败不应阻止用户重新检查更新。
    } finally {
      pendingUpdate = null
    }
  }

  return {
    phase,
    lastCheckResult,
    lastError,
    availableUpdate,
    isDialogOpen,
    isChecking,
    isInstalling,
    downloadProgress,
    checkForUpdates,
    installUpdate,
    dismissUpdate,
    openUpdateDialog,
    openReleasesPage
  }
})
