import { afterEach, describe, expect, it, vi } from 'vitest'
import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { useGamesStore } from '../modules/games/stores/games.ts'
import { useAppUpdaterStore } from '../modules/updates/stores/appUpdater'
import { routeNames } from '../router/routeNames'
import { nextTick } from 'vue'
import { useToast } from '../shared/composables/useToast'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import ScanResultsDialog from '../modules/games/components/ScanResultsDialog.vue'

//  components
import GameLibraryLayout from './GameLibraryLayout.vue'
import AddGameDialog from '../modules/games/components/AddGameDialog.vue'
import AllGamesPage from '../pages/AllGamesPage.vue'
import RemoveGameDialog from '../modules/games/components/RemoveGameDialog.vue'
// type
import type { CreateGamePayload, Game, ScanCandidate, UpdateGamePayload } from '../modules/games/types/game.ts'

enableAutoUnmount(afterEach)

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

const TestPage = {
  template: '<div>Test page</div>'
}

const createPayload: CreateGamePayload = {
  name: '空洞骑士',
  exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
  workDir: 'D:\\Games\\Hollow Knight',
  args: null,
  coverPath: null,
  coverSelection: {
    type: 'unchanged'
  }
}

const createdGame: Game = {
  id: 'game-1',
  name: createPayload.name,
  exePath: createPayload.exePath,
  folderPath: 'D:\\Games\\Hollow Knight',
  workDir: createPayload.workDir,
  args: null,
  favorite: false,
  playCount: 0,
  createTime: 1000,
  updateTime: 1000
}

function createTestRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/home', name: routeNames.home, component: TestPage },
      { path: '/games', name: routeNames.games, component: AllGamesPage },
      { path: '/favorites', name: routeNames.favorites, component: TestPage },
      { path: '/recent', name: routeNames.recent, component: TestPage },
      { path: '/settings', name: routeNames.settings, component: TestPage }
    ]
  })
}

async function initGameLibraryLayout(path: string = '/home') {
  const pinia = createPinia()
  const gamesStore = useGamesStore(pinia)
  const appUpdaterStore = useAppUpdaterStore(pinia)
  // 监听并可替换某个对象方法
  const loadGamesSpy = vi.spyOn(gamesStore, 'loadGames').mockResolvedValue()
  const automaticCheckSpy = vi.spyOn(appUpdaterStore, 'runAutomaticCheck').mockResolvedValue()

  const router = createTestRouter()
  await router.push(path)
  await router.isReady()

  const wrapper = mount(GameLibraryLayout, {
    global: {
      plugins: [pinia, router]
    }
  })
  await flushPromises()
  return {
    wrapper,
    router,
    gamesStore,
    appUpdaterStore,
    loadGamesSpy,
    automaticCheckSpy
  }
}

describe('GameLibraryLayout', () => {
  //  ================================================ add game
  // 测试加载游戏
  it('loads games when mounted', async () => {
    const { loadGamesSpy } = await initGameLibraryLayout()

    expect(loadGamesSpy).toHaveBeenCalledTimes(1)
  })

  it('checks app updates when mounted', async () => {
    const { automaticCheckSpy } = await initGameLibraryLayout()

    expect(automaticCheckSpy).toHaveBeenCalledTimes(1)
  })

  it('opens the add game dialog when the add button is clicked', async () => {
    const { wrapper } = await initGameLibraryLayout('/games')

    const buttons = wrapper.findAll('button')
    const addButton = buttons.find((button) => button.text().trim() === '添加游戏')
    expect(addButton).toBeDefined()
    await addButton!.trigger('click')

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog?.textContent).toContain('手动添加游戏')
    expect(dialog?.textContent).not.toContain('保存修改')
  })

  it('creates a game when the add dialog is submitted', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout()
    const addButton = wrapper.findAll('button').find((button) => button.text().trim() === '添加游戏')
    expect(addButton).toBeDefined()
    await addButton!.trigger('click')

    const createGameSpy = vi.spyOn(gamesStore, 'createGame').mockResolvedValue(createdGame)

    // 打开弹框， 共提交
    const addGameDialog = wrapper.getComponent(AddGameDialog)
    expect(addGameDialog.props('open')).toBe(true)
    expect(addGameDialog.props('mode')).toBe('create')
    addGameDialog.vm.$emit('submit', createPayload)
    await flushPromises()

    expect(createGameSpy).toHaveBeenCalledTimes(1)
    expect(createGameSpy).toHaveBeenCalledWith(createPayload)
    expect(addGameDialog.props('open')).toBe(false)
  })

  it('keeps the add game dialog open when creating a game fails', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout()
    const addButton = wrapper.findAll('button').find((button) => button.text().trim() === '添加游戏')
    expect(addButton).toBeDefined()
    await addButton!.trigger('click')

    const createGameSpy = vi.spyOn(gamesStore, 'createGame').mockRejectedValue(new Error('新增失败'))

    // 打开弹框， 共提交
    const addGameDialog = wrapper.getComponent(AddGameDialog)
    addGameDialog.vm.$emit('submit', createPayload)
    await flushPromises()

    expect(createGameSpy).toHaveBeenCalledTimes(1)
    expect(createGameSpy).toHaveBeenCalledWith(createPayload)

    expect(addGameDialog.props('open')).toBe(true)
  })

  //  ================================================ edit
  it('opens the edit dialog for the selected game', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [createdGame]
    await nextTick()

    const editButton = wrapper.find('button[aria-label="编辑游戏"]')
    expect(editButton.exists()).toBe(true)
    await editButton.trigger('click')

    const addGameDialog = wrapper.getComponent(AddGameDialog)
    expect(addGameDialog.props('open')).toBe(true)
    expect(addGameDialog.props('mode')).toBe('edit')
    expect(addGameDialog.props('game')).toEqual(createdGame)
  })

  it('updates a game when the edit dialog is submitted', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [{ ...createdGame, name: '王者' }]
    await nextTick()

    const updateGameSpy = vi.spyOn(gamesStore, 'updateGame').mockResolvedValue(createdGame)

    const editButton = wrapper.find('button[aria-label="编辑游戏"]')
    expect(editButton.exists()).toBe(true)
    await editButton.trigger('click')

    const addGameDialog = wrapper.getComponent(AddGameDialog)
    const updatePayload: UpdateGamePayload = {
      ...createPayload,
      id: createdGame.id,
      favorite: createdGame.favorite
    }

    addGameDialog.vm.$emit('submit', updatePayload)
    await flushPromises()

    expect(updateGameSpy).toHaveBeenCalledTimes(1)
    expect(updateGameSpy).toHaveBeenCalledWith(updatePayload)
    expect(addGameDialog.props('open')).toBe(false)
  })

  it('keeps dialog open when update game fails', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const updateGameSpy = vi.spyOn(gamesStore, 'updateGame').mockRejectedValue(new Error('update fail'))
    gamesStore.games = [{ ...createdGame, name: '王者' }]
    await nextTick()

    const editButton = wrapper.find('button[aria-label="编辑游戏"]')
    expect(editButton.exists()).toBe(true)
    await editButton.trigger('click')

    const addGameDialog = wrapper.getComponent(AddGameDialog)
    expect(addGameDialog.props('open')).toBe(true)
    const updatePayload: UpdateGamePayload = {
      ...createPayload,
      id: createdGame.id,
      favorite: createdGame.favorite
    }
    addGameDialog.vm.$emit('submit', updatePayload)
    await flushPromises()

    expect(updateGameSpy).toHaveBeenCalledTimes(1)
    expect(updateGameSpy).toHaveBeenCalledWith(updatePayload)
    expect(addGameDialog.props('open')).toBe(true)
  })

  //  ================================================ delete game
  it('open the delete game dialog when the delete button is clicked', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [createdGame]
    await nextTick()

    const deleteButton = wrapper.find('button[aria-label="移除游戏"]')
    expect(deleteButton.exists()).toBe(true)
    await deleteButton!.trigger('click')

    const deleteDialog = wrapper.getComponent(RemoveGameDialog)
    expect(deleteDialog.props('open')).toBe(true)
    expect(deleteDialog.props('game')).toEqual(createdGame)

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog?.textContent).toContain('移除游戏')
    expect(dialog?.textContent).toContain(createdGame.name)
  })

  it('deletes a game when the delete dialog is confirmed', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const deleteGameSpy = vi.spyOn(gamesStore, 'deleteGame').mockResolvedValue()
    gamesStore.games = [createdGame]
    await nextTick()

    const deleteButton = wrapper.find('button[aria-label="移除游戏"]')
    expect(deleteButton.exists()).toBe(true)
    await deleteButton!.trigger('click')

    const deleteDialog = wrapper.getComponent(RemoveGameDialog)
    deleteDialog.vm.$emit('confirm')
    await flushPromises()

    expect(deleteGameSpy).toHaveBeenCalledTimes(1)
    expect(deleteGameSpy).toHaveBeenCalledWith(createdGame.id)
    expect(deleteDialog.props('open')).toBe(false)
  })

  it('keeps delete dialog open when delete game fails', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const deleteGameSpy = vi.spyOn(gamesStore, 'deleteGame').mockRejectedValue(new Error('delete fail'))
    gamesStore.games = [createdGame]
    await nextTick()

    const deleteButton = wrapper.find('button[aria-label="移除游戏"]')
    expect(deleteButton.exists()).toBe(true)
    await deleteButton!.trigger('click')

    const deleteDialog = wrapper.getComponent(RemoveGameDialog)
    expect(deleteDialog.props('open')).toBe(true)
    deleteDialog.vm.$emit('confirm')
    await flushPromises()

    expect(deleteGameSpy).toHaveBeenCalledTimes(1)
    expect(deleteGameSpy).toHaveBeenCalledWith(createdGame.id)
    expect(deleteDialog.props('open')).toBe(true)
  })

  //   ========================================= favorite
  it('collects game when the favorite button is clicked', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [createdGame]
    const updateGameSpy = vi.spyOn(gamesStore, 'updateGame').mockResolvedValue({ ...createdGame, favorite: true })
    await nextTick()

    const favoriteButton = wrapper.find('button[aria-label="收藏游戏"]')
    expect(favoriteButton.exists()).toBe(true)
    await favoriteButton!.trigger('click')

    const payload: UpdateGamePayload = {
      id: createdGame.id,
      name: createdGame.name,
      exePath: createdGame.exePath,
      workDir: createdGame.workDir,
      args: createdGame.args,
      favorite: true
    }
    expect(updateGameSpy).toHaveBeenCalledTimes(1)
    expect(updateGameSpy).toHaveBeenCalledWith(payload)
  })

  it('removes a game from favorites when the favorite button is clicked', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [{ ...createdGame, favorite: true }]
    const updateGameSpy = vi.spyOn(gamesStore, 'updateGame').mockResolvedValue({ ...createdGame })
    await nextTick()

    const favoriteButton = wrapper.find('button[aria-label="取消收藏"]')
    expect(favoriteButton.exists()).toBe(true)
    await favoriteButton!.trigger('click')

    const payload: UpdateGamePayload = {
      id: createdGame.id,
      name: createdGame.name,
      exePath: createdGame.exePath,
      workDir: createdGame.workDir,
      args: createdGame.args,
      favorite: false
    }
    expect(updateGameSpy).toHaveBeenCalledTimes(1)
    expect(updateGameSpy).toHaveBeenCalledWith(payload)
  })

  it('opens tip when the favorite status update fails', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    gamesStore.games = [createdGame]
    const updateGameSpy = vi.spyOn(gamesStore, 'updateGame').mockRejectedValue(new Error('update favorite status fail'))
    await nextTick()

    const favoriteButton = wrapper.find('button[aria-label="收藏游戏"]')
    expect(favoriteButton.exists()).toBe(true)
    await favoriteButton!.trigger('click')
    expect(updateGameSpy).toHaveBeenCalledTimes(1)

    await flushPromises()

    const toast = useToast()
    const latestToast = toast.toasts.value[toast.toasts.value.length - 1]
    expect(latestToast).toMatchObject({
      type: 'error',
      title: '更新收藏状态失败',
      description: 'update favorite status fail'
    })
  })

  //   =========================== launch
  it('launches a game when the launch button is clicked', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const launchGameSpy = vi.spyOn(gamesStore, 'launchGame').mockResolvedValue(createdGame)
    gamesStore.games = [createdGame]
    await nextTick()

    const launchButton = wrapper.find('button[aria-label="启动游戏"]')
    expect(launchButton.exists()).toBe(true)
    await launchButton!.trigger('click')

    expect(launchGameSpy).toHaveBeenCalledTimes(1)
    expect(launchGameSpy).toHaveBeenCalledWith(createdGame.id)
  })

  it('opens tip when launch game fails', async () => {
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const launchGameSpy = vi.spyOn(gamesStore, 'launchGame').mockRejectedValue(new Error('launch game fail'))
    gamesStore.games = [createdGame]
    await nextTick()

    const launchButton = wrapper.find('button[aria-label="启动游戏"]')
    expect(launchButton.exists()).toBe(true)
    await launchButton!.trigger('click')

    expect(launchGameSpy).toHaveBeenCalledTimes(1)
    expect(launchGameSpy).toHaveBeenCalledWith(createdGame.id)

    await flushPromises()
    const toast = useToast()
    const latestToast = toast.toasts.value[toast.toasts.value.length - 1]
    expect(latestToast).toMatchObject({
      type: 'error',
      title: '启动游戏失败',
      description: 'launch game fail'
    })
  })

  // ==================================== scan folder
  it('does not open the scan dialog when folder selection is cancelled', async () => {
    vi.mocked(openDialog).mockResolvedValue(null)
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')
    const scanGamesSpy = vi.spyOn(gamesStore, 'scanGames')

    gamesStore.games = [createdGame]
    await nextTick()

    const buttons = wrapper.findAll('button')
    const scanButton = buttons.find((button) => button.text().trim() === '扫描目录')
    expect(scanButton).toBeDefined()
    await scanButton!.trigger('click')
    await flushPromises()

    // 1. 打开了文件选择器
    expect(openDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: true
    })
    // 2. 没有调用扫描
    expect(scanGamesSpy).not.toHaveBeenCalled()
    // 3. 不打开弹框
    const scanDialog = wrapper.getComponent(ScanResultsDialog)
    expect(scanDialog.props('open')).toBe(false)
  })

  it('opens the scan dialog when a folder is selected', async () => {
    const selectedDirectory = 'D:\\Games'
    vi.mocked(openDialog).mockResolvedValue(selectedDirectory)
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')

    const scanCandidate: ScanCandidate = {
      name: 'games',
      exePath: 'D:\\Games\\test.exe',
      folderPath: 'D:\\Games',
      exeFileName: 'Games/test.exe',
      exists: true,
      recommended: true,
      confidence: 1,
      reasons: ['test.exe']
    }
    const scanGamesSpy = vi.spyOn(gamesStore, 'scanGames').mockResolvedValue([scanCandidate])
    await nextTick()

    // 点击按钮
    const buttons = wrapper.findAll('button')
    const scanButton = buttons.find((button) => button.text().trim() === '扫描目录')
    expect(scanButton).toBeDefined()
    await scanButton!.trigger('click')
    await flushPromises()

    // 1. 打开了文件选择器
    expect(openDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: true
    })
    // 2. 扫描目录
    expect(scanGamesSpy).toHaveBeenCalledTimes(1)
    expect(scanGamesSpy).toHaveBeenCalledWith(selectedDirectory)

    // 3. 打开弹框
    const scanDialog = wrapper.getComponent(ScanResultsDialog)
    expect(scanDialog.props('open')).toBe(true)
    expect(scanDialog.props('candidates')).toEqual([scanCandidate])
  })

  it('keeps the scan dialog open and show an error when scanning fails ', async () => {
    const selectedDirectory = 'D:\\Games'
    vi.mocked(openDialog).mockResolvedValue(selectedDirectory)
    const { wrapper, gamesStore } = await initGameLibraryLayout('/games')

    const scanGamesSpy = vi.spyOn(gamesStore, 'scanGames').mockRejectedValue(new Error('scan game fail'))
    await nextTick()

    // 点击按钮
    const buttons = wrapper.findAll('button')
    const scanButton = buttons.find((button) => button.text().trim() === '扫描目录')
    expect(scanButton).toBeDefined()
    await scanButton!.trigger('click')
    await flushPromises()

    // 1. 扫描目录
    expect(scanGamesSpy).toHaveBeenCalledTimes(1)
    expect(scanGamesSpy).toHaveBeenCalledWith(selectedDirectory)

    // 2. 打开弹框
    const scanDialog = wrapper.getComponent(ScanResultsDialog)
    expect(scanDialog.props('open')).toBe(true)
    expect(scanDialog.props('errorMessage')).toBe('scan game fail')
  })
})
