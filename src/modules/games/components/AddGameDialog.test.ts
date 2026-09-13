import { nextTick } from 'vue'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { describe, afterEach, it, expect, vi } from 'vitest'
import { mount, enableAutoUnmount, flushPromises } from '@vue/test-utils'
import AddGameDialog from './AddGameDialog.vue'
// type
import type { CreateGamePayload, Game, UpdateGamePayload } from '../types/game'

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

enableAutoUnmount(afterEach)

const game: Game = {
  id: 'game-1',
  name: '空洞骑士',
  exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
  folderPath: 'D:\\Games\\Hollow Knight',
  workDir: 'D:\\Games\\Hollow Knight',
  args: '-windowed',
  favorite: false,
  playCount: 3,
  createTime: 1000,
  updateTime: 2000
}

describe('AddGameDialog', () => {
  it('renders create mode title and actions', async () => {
    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).toContain('手动添加游戏')
    expect(dialog?.textContent).toContain('本地')
    expect(dialog?.textContent).toContain('保存')
    expect(dialog?.textContent).not.toContain('保存修改')
  })

  it('shows an error and does not submit without an executable', async () => {
    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const buttonArr = Array.from(document.body.querySelectorAll('button'))
    const button = buttonArr.find((el) => el.textContent?.trim() === '保存')
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('submit')).toBeUndefined()

    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).toContain('请选择游戏启动程序')
  })

  it('emits the create payload when then submit button is clicked', async () => {
    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    const payload: CreateGamePayload = {
      name: '王者荣耀',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
      workDir: null,
      args: null,
      coverPath: null,
      coverSelection: {
        type: 'unchanged'
      }
    }
    await nextTick()

    const gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    expect(gameNameInput).not.toBeNull()
    gameNameInput!.value = payload.name
    gameNameInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const gameExeInput = document.body.querySelector<HTMLInputElement>('#game-exe')
    expect(gameExeInput).not.toBeNull()
    gameExeInput!.value = payload.exePath
    gameExeInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const buttonArr = Array.from(document.body.querySelectorAll('button'))
    const button = buttonArr.find((el) => el.textContent?.trim() === '保存')
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('submit')).toEqual([[payload]])
  })

  it('prevents submit when the game path does not end with exe', async () => {
    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    expect(gameNameInput).not.toBeNull()
    gameNameInput!.value = '王者荣耀'
    gameNameInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const gameExeInput = document.body.querySelector<HTMLInputElement>('#game-exe')
    expect(gameExeInput).not.toBeNull()
    gameExeInput!.value = 'D:\\Games\\Hollow Knight\\hollow_knight.txt'
    gameExeInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const buttonArr = Array.from(document.body.querySelectorAll('button'))
    const button = buttonArr.find((el) => el.textContent?.trim() === '保存')
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('submit')).toBeUndefined()

    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).include('启动程序必须是 .exe 文件')
  })

  it('does not submit when the game name is blank', async () => {
    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    expect(gameNameInput).not.toBeNull()
    gameNameInput!.value = ''
    gameNameInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const gameExeInput = document.body.querySelector<HTMLInputElement>('#game-exe')
    expect(gameExeInput).not.toBeNull()
    gameExeInput!.value = 'D:\\Games\\Hollow Knight\\hollow_knight.exe'
    gameExeInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const buttonArr = Array.from(document.body.querySelectorAll('button'))
    const button = buttonArr.find((el) => el.textContent?.trim() === '保存')
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('submit')).toBeUndefined()

    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).include('游戏名称不能为空')
  })

  it('fills the game name and work directory after selecting an executable', async () => {
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight\\hollow_knight.exe')

    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const selectButtons = Array.from(document.body.querySelectorAll('button')).filter(
      (button) => button.textContent?.trim() === '选择'
    )
    expect(selectButtons).toHaveLength(2)
    selectButtons[0].click()
    await flushPromises()

    expect(openDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: 'Windows 可执行文件', extensions: ['exe'] }]
    })
    expect(document.body.querySelector<HTMLInputElement>('#game-name')?.value).toBe('Hollow Knight')
    expect(document.body.querySelector<HTMLInputElement>('#game-exe')?.value).toBe(
      'D:\\Games\\Hollow Knight\\hollow_knight.exe'
    )
    expect(document.body.querySelector<HTMLInputElement>('#game-work-dir')?.value).toBe('D:\\Games\\Hollow Knight')
  })

  it('updates the work directory after selecting a folder', async () => {
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight')

    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const selectButtons = Array.from(document.body.querySelectorAll('button')).filter(
      (button) => button.textContent?.trim() === '选择'
    )
    expect(selectButtons).toHaveLength(2)
    selectButtons[1].click()
    await flushPromises()

    expect(openDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: true,
      defaultPath: undefined
    })
    expect(document.body.querySelector<HTMLInputElement>('#game-name')?.value).toBe('')
    expect(document.body.querySelector<HTMLInputElement>('#game-exe')?.value).toBe('')
    expect(document.body.querySelector<HTMLInputElement>('#game-work-dir')?.value).toBe('D:\\Games\\Hollow Knight')
  })

  it('selected cover when create new game', async () => {
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight\\hollow_knight.png')

    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          GameArtwork: true
        }
      },
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    const selectButtons = Array.from(document.body.querySelectorAll('button'))
    const selectCoverButton = selectButtons.find((button) => button.textContent?.trim() === '本地')
    expect(selectCoverButton).toBeDefined()
    selectCoverButton?.click()
    await flushPromises()

    expect(openDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: '游戏封面', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]
    })

    // 补充其他必填数据
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight\\hollow_knight.exe')
    const otherButtons = selectButtons.filter((button) => button.textContent?.trim() === '选择')
    expect(otherButtons).toHaveLength(2)
    otherButtons[0].click()
    await flushPromises()

    const submitButton = selectButtons.find((button) => button.textContent.trim() === '保存')
    expect(submitButton).toBeDefined()
    submitButton?.click()
    const payload: CreateGamePayload = {
      name: 'Hollow Knight',
      exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
      workDir: 'D:\\Games\\Hollow Knight',
      args: null,
      coverPath: null,
      coverSelection: {
        type: 'local',
        path: 'D:\\Games\\Hollow Knight\\hollow_knight.png'
      }
    }
    expect(wrapper.emitted('submit')).toEqual([[payload]])
  })

  it('resets from data when the dialog is closed and reopened', async () => {
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight\\hollow_knight.exe')

    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create'
      }
    })
    await nextTick()

    // 补充其他必填数据
    const selectButtons = Array.from(document.body.querySelectorAll('button'))
    const otherButtons = selectButtons.filter((button) => button.textContent?.trim() === '选择')
    expect(otherButtons).toHaveLength(2)
    otherButtons[0].click()
    await flushPromises()

    let gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    expect(gameNameInput).not.toBeNull()
    expect(gameNameInput!.value).toBe('Hollow Knight')

    // 关闭弹窗
    const closeButton = selectButtons.find((button) => button.textContent?.trim() === '取消')
    closeButton?.click()
    expect(wrapper.emitted('close')).toEqual([[]])
    await wrapper.setProps({ open: false })
    await nextTick()

    // 重新打开
    await wrapper.setProps({ open: true })
    gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    expect(gameNameInput).not.toBeNull()
    expect(gameNameInput!.value).toBe('')
  })

  it('shows the error message when creating a game fails', async () => {
    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'create',
        errorMessage: '该游戏启动路径已存在'
      }
    })
    await nextTick()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog?.textContent).toContain('该游戏启动路径已存在')
  })

  //   编辑游戏 ======================================
  it('renders edit mode title and actions', async () => {
    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'edit',
        game
      }
    })
    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).toContain('编辑游戏')
    expect(dialog?.textContent).toContain('本地')
    expect(dialog?.textContent).toContain('保存修改')
    expect(dialog?.textContent).not.toContain('手动添加游戏')
  })

  it('fills game data in the dialog', async () => {
    mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'edit',
        game
      }
    })
    await nextTick()
    const gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    const gameExeInput = document.body.querySelector<HTMLInputElement>('#game-exe')
    const gameWorkDirInput = document.body.querySelector<HTMLInputElement>('#game-work-dir')
    const gameArgsInput = document.body.querySelector<HTMLInputElement>('#game-args')

    expect(gameNameInput?.value).toBe(game.name)
    expect(gameExeInput?.value).toBe(game.exePath)
    expect(gameWorkDirInput?.value).toBe(game.workDir)
    expect(gameArgsInput?.value).toBe(game.args)
  })

  it('emits payload is correctly', async () => {
    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'edit',
        game
      }
    })
    await nextTick()

    // 编辑数据
    const gameNameInput = document.body.querySelector<HTMLInputElement>('#game-name')
    const gameExeInput = document.body.querySelector<HTMLInputElement>('#game-exe')
    const gameWorkDirInput = document.body.querySelector<HTMLInputElement>('#game-work-dir')
    const gameArgsInput = document.body.querySelector<HTMLInputElement>('#game-args')
    const updateData = {
      name: '你好啊',
      exePath: 'C:\\Games\\Hollow Knight\\test.exe',
      workDir: 'C:\\Games\\Hollow Knight',
      args: ''
    }
    gameNameInput!.value = updateData.name
    gameExeInput!.value = updateData.exePath
    gameWorkDirInput!.value = updateData.workDir
    gameArgsInput!.value = updateData.args

    gameNameInput!.dispatchEvent(new Event('input', { bubbles: true }))
    gameExeInput!.dispatchEvent(new Event('input', { bubbles: true }))
    gameWorkDirInput!.dispatchEvent(new Event('input', { bubbles: true }))
    gameArgsInput!.dispatchEvent(new Event('input', { bubbles: true }))

    const buttons = Array.from(document.querySelectorAll('button'))
    const submitButton = buttons.find((button) => button.textContent.trim() === '保存修改')
    expect(submitButton).toBeDefined()
    submitButton?.click()

    const expectedPayload: UpdateGamePayload = {
      id: game.id,
      favorite: game.favorite,
      name: updateData.name,
      exePath: updateData.exePath,
      workDir: updateData.workDir,
      args: null,
      coverPath: null,
      coverSelection: {
        type: 'unchanged'
      }
    }

    expect(wrapper.emitted('submit')).toEqual([[expectedPayload]])
  })

  it('does not submit when edit mode has no game', async () => {
    vi.mocked(openDialog).mockResolvedValue('D:\\Games\\Hollow Knight\\hollow_knight.exe')

    const wrapper = mount(AddGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        saving: false,
        mode: 'edit'
      }
    })
    await nextTick()

    //   手动补齐的表单数据
    const buttons = Array.from(document.body.querySelectorAll('button'))

    const selectButton = buttons.filter((button) => button.textContent.trim() === '选择')
    const selectExeButton = selectButton[0]
    expect(selectExeButton).toBeDefined()
    selectExeButton?.click()
    await flushPromises()

    const submitButton = buttons.find((button) => button.textContent.trim() === '保存修改')
    expect(submitButton).toBeDefined()
    submitButton?.click()
    expect(wrapper.emitted('submit')).toBeUndefined()

    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).toContain('缺少要编辑的游戏记录')
  })
})
