import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setAutoCheckUpdatesEnabled } from '../../settings/api.ts'
import { useAppUpdaterStore } from './appUpdater.ts'
import { createPinia, setActivePinia } from 'pinia'
import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { flushPromises } from '@vue/test-utils'

vi.mock('../../settings/api', () => ({
  getAppUpdateSettings: vi.fn(),
  setAutoCheckUpdatesEnabled: vi.fn()
}))
vi.mock('@tauri-apps/plugin-updater', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@tauri-apps/plugin-updater')>()),
  check: vi.fn()
}))
vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn()
}))

describe('updates app updater', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('saves and updates the automatic check setting', async () => {
    vi.mocked(setAutoCheckUpdatesEnabled).mockResolvedValueOnce({
      autoCheckEnabled: false
    })
    const store = useAppUpdaterStore()
    store.settings = {
      autoCheckEnabled: true
    }
    await store.setAutoCheckEnabled(false)

    expect(setAutoCheckUpdatesEnabled).toHaveBeenCalledTimes(1)
    expect(setAutoCheckUpdatesEnabled).toHaveBeenNthCalledWith(1, false)
    expect(store.settings).toEqual({ autoCheckEnabled: false })
    expect(store.autoCheckEnabled).toBe(false)
    expect(store.settingsError).toBeNull()
  })

  it('preserves the previous setting and reports a save failure', async () => {
    vi.mocked(setAutoCheckUpdatesEnabled).mockRejectedValueOnce(new Error('save error'))

    const store = useAppUpdaterStore()
    store.settings = { autoCheckEnabled: true }
    await expect(store.setAutoCheckEnabled(false)).rejects.toThrow('save error')

    expect(setAutoCheckUpdatesEnabled).toHaveBeenCalledTimes(1)
    expect(setAutoCheckUpdatesEnabled).toHaveBeenCalledWith(false)
    expect(store.settings).toEqual({ autoCheckEnabled: true })
    expect(store.autoCheckEnabled).toBe(true)
    expect(store.settingsError).toBe('save error')
  })

  it('returns to idle when no update is available', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    vi.mocked(check).mockResolvedValueOnce(null)

    const store = useAppUpdaterStore()

    const request = store.checkForUpdates('automatic')
    expect(store.phase).toBe('checking')
    await request

    expect(check).toHaveBeenCalledTimes(1)
    expect(store.phase).toBe('idle')
    expect(store.lastCheckResult).toBe('upToDate')
    expect(store.availableUpdate).toBeNull()
    expect(store.isDialogOpen).toBe(false)
    expect(store.lastError).toBeNull()
  })

  it('opens the update dialog when a new version is available', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const update = new Update({
      rid: 1,
      currentVersion: '0.2.3',
      version: '0.2.4',
      date: '2026-10-16',
      body: '修复问题',
      rawJson: {}
    })

    vi.mocked(check).mockResolvedValueOnce(update)

    const store = useAppUpdaterStore()

    const request = store.checkForUpdates('automatic')
    expect(store.phase).toBe('checking')
    await request

    expect(check).toHaveBeenCalledTimes(1)
    expect(store.phase).toBe('available')
    expect(store.lastCheckResult).toBe('available')
    expect(store.availableUpdate).toEqual({
      currentVersion: '0.2.3',
      version: '0.2.4',
      date: '2026-10-16',
      notes: '修复问题'
    })
    expect(store.isDialogOpen).toBe(true)
    expect(store.lastError).toBeNull()
  })

  it('recovers from a failed update check on retry', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const update = new Update({
      rid: 1,
      currentVersion: '0.2.3',
      version: '0.2.4',
      date: '2026-10-16',
      body: '修复问题',
      rawJson: {}
    })
    vi.mocked(check).mockRejectedValueOnce(new Error('check error')).mockResolvedValueOnce(update)

    const store = useAppUpdaterStore()
    await store.checkForUpdates('automatic')
    expect(store.phase).toBe('idle')
    expect(store.lastCheckResult).toBe('failed')
    expect(store.lastError).toBe('check error')
    expect(store.isDialogOpen).toBe(false)

    await store.checkForUpdates('automatic')
    expect(check).toHaveBeenCalledTimes(2)
    expect(store.phase).toBe('available')
    expect(store.lastCheckResult).toBe('available')
    expect(store.availableUpdate?.version).toBe('0.2.4')
    expect(store.lastError).toBeNull()
    expect(store.isDialogOpen).toBe(true)
  })

  // 测试场景：总大小为 100 字节，依次收到 30、20 字节，验证 Store 将进度累加为 30%、50%。
  // 安装和重启在本用例中只模拟成功，不验证真实安装能力或各阶段的等待顺序。
  it('downloads successfully', async () => {
    // 模拟 Tauri 环境标记，让 Store 通过环境检查；并没有启动真实桌面环境。
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    // 创建更新信息对象；构造对象本身不会下载或安装，下面会替换它的相关方法。
    const update = new Update({
      rid: 1,
      // 当前已安装版本。
      currentVersion: '0.2.3',
      // 检查接口发现的新版本。
      version: '0.2.4',
      date: '2026-10-16',
      body: '修复问题',
      rawJson: {}
    })
    // check 已在文件顶部被 mock；指定下一次调用异步返回 update，不进行真实联网检查。
    vi.mocked(check).mockResolvedValueOnce(update)

    // 获取当前测试的 Store 实例，执行真实的 Store 业务逻辑。
    const store = useAppUpdaterStore()
    // 调用 mock 的 check，将 update 保存为待安装更新；automatic 避免手动检查的 Toast。
    await store.checkForUpdates('automatic')
    expect(store.lastCheckResult).toBe('available')

    // mock ①：替换此 update 实例的 download 方法，并保留调用记录，不下载真实文件。
    // 此处只配置替代函数；等下面 installUpdate 调用 download 时，函数体才执行。
    // onEvent 是 Store 传给 download 的真实进度处理回调，由测试模拟下载器来调用。
    vi.spyOn(update, 'download').mockImplementation(async (onEvent) => {
      onEvent?.({ event: 'Started', data: { contentLength: 100 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 30 } })
      expect(store.downloadProgress).toBe(30)

      onEvent?.({ event: 'Progress', data: { chunkLength: 20 } })
    })

    // mock ②：替换此 update 的 install 方法，返回成功且无返回值的 Promise，不安装软件。
    vi.spyOn(update, 'install').mockResolvedValue(undefined)
    // mock ③：relaunch 已在文件顶部被 mock；配置它异步成功，不重启当前进程。
    // vi.mocked 只辅助类型推断，真正将 relaunch 替换为 mock 的是顶部的 vi.mock。
    vi.mocked(relaunch).mockResolvedValue(undefined)

    // 执行真实 Store 流程：调用 mock 下载并处理进度 → mock 安装 → mock 重启，等待整个流程完成。
    await store.installUpdate()
    // 验证两次数据块被累加：(30 + 20) / 100 × 100 = 50%，而不是覆盖成 20%。
    expect(store.downloadProgress).toBe(50)
    expect(store.lastError).toBeNull()
  })

  it('waits for download and installation before restarting', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    // 创建更新信息对象；构造对象本身不会下载或安装，下面会替换它的相关方法。
    const update = new Update({
      rid: 1,
      // 当前已安装版本。
      currentVersion: '0.2.3',
      // 检查接口发现的新版本。
      version: '0.2.4',
      date: '2026-10-16',
      body: '修复问题',
      rawJson: {}
    })
    vi.mocked(check).mockResolvedValueOnce(update)

    const store = useAppUpdaterStore()
    await store.checkForUpdates('automatic')
    expect(store.lastCheckResult).toBe('available')

    let finishDownload!: () => void
    const downloading = new Promise<void>((resolve) => {
      finishDownload = resolve
    })

    let finishInstall!: () => void
    const installing = new Promise<void>((resolve) => {
      finishInstall = resolve
    })

    vi.spyOn(update, 'download').mockReturnValue(downloading)
    vi.spyOn(update, 'install').mockReturnValue(installing)
    vi.mocked(relaunch).mockResolvedValue(undefined)

    // 1. 下载尚未完成：不能开始安装
    const task = store.installUpdate()
    expect(store.phase).toBe('downloading')
    expect(update.download).toHaveBeenCalledTimes(1)
    expect(update.install).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()

    // 2. 下载完成、安装尚未完成：不能重启
    finishDownload()
    await flushPromises()
    expect(store.phase).toBe('installing')
    expect(update.install).toHaveBeenCalledTimes(1)
    expect(relaunch).not.toHaveBeenCalled()

    // 3. 安装完成：触发重启
    finishInstall()
    await task
    expect(store.phase).toBe('restarting')
    expect(relaunch).toHaveBeenCalledTimes(1)
    expect(store.lastError).toBeNull()
  })

  // 测试场景：下载失败后记录错误、解除处理中状态，不继续安装；重试时清除旧进度和错误并完成更新。
  it('recovers from a download failure on retry', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const update = new Update({ rid: 1, currentVersion: '0.2.3', version: '0.2.4', rawJson: {} })
    vi.mocked(check).mockResolvedValueOnce(update)
    const download = vi.spyOn(update, 'download').mockImplementationOnce(async (onEvent) => {
      onEvent?.({ event: 'Started', data: { contentLength: 100 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 30 } })
      throw new Error('下载中断')
    })
    const install = vi.spyOn(update, 'install').mockResolvedValue(undefined)
    vi.mocked(relaunch).mockResolvedValue(undefined)
    const store = useAppUpdaterStore()
    await store.checkForUpdates('automatic')

    await store.installUpdate()
    expect(store.phase).toBe('failed')
    expect(store.lastError).toBe('下载中断')
    expect(store.isInstalling).toBe(false)
    expect(store.downloadProgress).toBe(30)
    expect(install).not.toHaveBeenCalled()
    expect(relaunch).not.toHaveBeenCalled()

    // 暂停第二次下载，观察重试启动时是否清除了旧错误和累计进度。
    let finishDownload!: () => void
    download.mockReturnValueOnce(
      new Promise<void>((resolve) => {
        finishDownload = resolve
      })
    )
    const retry = store.installUpdate()
    expect(store.phase).toBe('downloading')
    expect(store.lastError).toBeNull()
    expect(store.downloadProgress).toBeNull()
    // 通过公开进度验证累计字节已清零，不访问 Store 内部未导出的变量。
    const retryProgress = download.mock.calls[1]![0]!
    retryProgress({ event: 'Started', data: { contentLength: 100 } })
    expect(store.downloadProgress).toBe(0)
    finishDownload()
    await retry

    expect(download).toHaveBeenCalledTimes(2)
    expect(install).toHaveBeenCalledTimes(1)
    expect(relaunch).toHaveBeenCalledTimes(1)
    expect(store.phase).toBe('restarting')
    expect(store.lastError).toBeNull()
  })

  // 测试场景：安装失败后不重启，解除处理中状态；用户重试后重新下载、安装并重启。
  it('recovers from an installation failure on retry', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const update = new Update({ rid: 1, currentVersion: '0.2.3', version: '0.2.4', rawJson: {} })
    vi.mocked(check).mockResolvedValueOnce(update)
    const download = vi.spyOn(update, 'download').mockResolvedValue(undefined)
    const install = vi
      .spyOn(update, 'install')
      .mockRejectedValueOnce(new Error('安装失败'))
      .mockResolvedValueOnce(undefined)
    vi.mocked(relaunch).mockResolvedValue(undefined)
    const store = useAppUpdaterStore()
    await store.checkForUpdates('automatic')

    await store.installUpdate()
    expect(download).toHaveBeenCalledTimes(1)
    expect(install).toHaveBeenCalledTimes(1)
    expect(store.phase).toBe('failed')
    expect(store.lastError).toBe('安装失败')
    expect(store.isInstalling).toBe(false)
    expect(relaunch).not.toHaveBeenCalled()

    await store.installUpdate()
    expect(download).toHaveBeenCalledTimes(2)
    expect(install).toHaveBeenCalledTimes(2)
    expect(relaunch).toHaveBeenCalledTimes(1)
    expect(store.phase).toBe('restarting')
    expect(store.lastError).toBeNull()
  })
})
