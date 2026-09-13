import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import RemoveGameDialog from './RemoveGameDialog.vue'
import { nextTick } from 'vue'

// type
import type { Game } from '../types/game.ts'

enableAutoUnmount(afterEach)

const game: Game = {
  id: 'game-1',
  name: '空洞骑士',
  exePath: 'D:\\Games\\Hollow Knight\\hollow_knight.exe',
  folderPath: 'D:\\Games\\Hollow Knight',
  favorite: false,
  playCount: 3,
  createTime: 1000,
  updateTime: 2000
}

describe('RemoveGameDialog', () => {
  it('renders the target game and safety message', async () => {
    mount(RemoveGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        game,
        deleting: false
      }
    })
    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog?.textContent).toContain('空洞骑士')
    expect(dialog?.textContent).toContain('D:\\Games\\Hollow Knight\\hollow_knight.exe')
    expect(dialog?.textContent).toContain('不会删除本地磁盘上的游戏文件。')
  })

  it('emits confirm when the remove button is clicked', async () => {
    const wrapper = mount(RemoveGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        game,
        deleting: false
      }
    })
    await nextTick()
    const button = Array.from(document.body.querySelectorAll<HTMLButtonElement>('button'))?.find(
      (el) => el.textContent?.trim() === '移除'
    )
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('confirm')).toEqual([[]])
  })

  it('emits close when the cancel button is clicked', async () => {
    const wrapper = mount(RemoveGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        game,
        deleting: false
      }
    })
    await nextTick()
    const button = Array.from(document.body.querySelectorAll<HTMLButtonElement>('button'))?.find(
      (el) => el.textContent?.trim() === '取消'
    )
    expect(button).toBeDefined()
    button?.click()
    expect(wrapper.emitted('close')).toEqual([[]])
    expect(wrapper.emitted('confirm')).toBeUndefined()
  })

  it('prevents emit while deleting game', async () => {
    const wrapper = mount(RemoveGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        game,
        deleting: true
      }
    })
    await nextTick()
    const buttons = Array.from(document.body.querySelectorAll<HTMLButtonElement>('button'))
    const cancelButton = buttons.find((button) => button.textContent?.trim() === '取消')
    const removeButton = buttons.find((button) => button.textContent?.trim() === '移除')

    expect(cancelButton).toBeDefined()
    expect(cancelButton?.disabled).toBe(true)
    cancelButton?.click()
    expect(wrapper.emitted('close')).toBeUndefined()

    expect(removeButton).toBeDefined()
    expect(removeButton?.disabled).toBe(true)
    removeButton?.click()
    expect(wrapper.emitted('confirm')).toBeUndefined()
  })

  it('render the errorMessage', async () => {
    mount(RemoveGameDialog, {
      attachTo: document.body,
      props: {
        open: true,
        game,
        deleting: false,
        errorMessage: '错误信息'
      }
    })
    await nextTick()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog?.textContent).toContain('错误信息')
  })
})
