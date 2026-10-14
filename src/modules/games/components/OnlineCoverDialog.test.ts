import { afterEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
import OnlineCoverDialog from './OnlineCoverDialog.vue'
import { nextTick } from 'vue'
import { listCoverCandidates, searchCoverCandidates } from '../api.ts'

import type { CoverCandidate, CoverSearchResult } from '../types/cover.ts'

enableAutoUnmount(afterEach)
vi.mock('../api', () => ({
  searchCoverCandidates: vi.fn(),
  listCoverCandidates: vi.fn()
}))

describe('onlineCoverDialog', () => {
  // 测试场景：空字符串或纯空格输入时显示校验提示，不发送搜索请求。
  it.each(['', '   '])('prevents search and shows an error for blank input %j', async (input) => {
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: {
        open: true,
        initialQuery: ''
      }
    })
    await nextTick()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog!.textContent).toContain('联网封面')

    const queryInput = dialog!.querySelector<HTMLInputElement>('input')
    expect(queryInput).not.toBeNull()
    await new DOMWrapper(queryInput!).setValue(input)

    const searchButton = dialog!.querySelector<HTMLButtonElement>('[type="submit"]')
    expect(searchButton).not.toBeNull()
    searchButton!.click()
    await flushPromises()

    expect(dialog!.textContent).toContain('请输入游戏名称后再搜索')
    expect(wrapper.emitted('select')).toBeUndefined()
    expect(searchCoverCandidates).not.toHaveBeenCalled()
  })

  it('searches with the trimmed initial query when opened and displays cover candidates', async () => {
    const coverSearchResult: CoverSearchResult = {
      matchedGame: {
        provider: 'steamgriddb',
        providerGameId: '1314',
        name: '空洞骑士'
      },
      alternativeGames: [],
      candidates: [
        {
          provider: 'steamgriddb',
          assetId: '11112222',
          providerGameId: '1314',
          previewUrl: 'https://aaa.ccc.png'
        }
      ]
    }
    vi.mocked(searchCoverCandidates).mockResolvedValue(coverSearchResult)

    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: {
        open: false,
        initialQuery: ' 空洞骑士 '
      }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()

    expect(searchCoverCandidates).toHaveBeenCalledTimes(1)
    expect(searchCoverCandidates).toHaveBeenCalledWith('空洞骑士')

    const matchAndSelectedGame = document.body.querySelector('[role="combobox"]')
    expect(matchAndSelectedGame).not.toBeNull()
    expect(matchAndSelectedGame!.textContent.trim()).toBe('空洞骑士')

    const firstImg = document.body.querySelector<HTMLImageElement>('img')
    expect(firstImg).not.toBeNull()
    expect(firstImg!.getAttribute('src')).toBe('https://aaa.ccc.png')
  })

  // 测试场景：首次搜索失败后恢复搜索按钮，重试成功后清除错误并显示封面候选。
  it('shows a search error and displays candidates after a successful retry', async () => {
    vi.mocked(searchCoverCandidates)
      .mockRejectedValueOnce(new Error('网络连接失败'))
      .mockResolvedValueOnce({
        matchedGame: { provider: 'steamgriddb', providerGameId: '1314', name: '空洞骑士' },
        alternativeGames: [],
        candidates: [
          {
            provider: 'steamgriddb',
            providerGameId: '1314',
            assetId: 'cover-1',
            previewUrl: 'https://example.com/cover.png'
          }
        ]
      })

    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '空洞骑士' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog!.textContent).toContain('网络连接失败')
    const searchButton = dialog!.querySelector<HTMLButtonElement>('[type="submit"]')
    expect(searchButton).not.toBeNull()
    expect(searchButton!.disabled).toBe(false)

    searchButton!.click()
    await flushPromises()

    expect(searchCoverCandidates).toHaveBeenCalledTimes(2)
    expect(searchCoverCandidates).toHaveBeenNthCalledWith(2, '空洞骑士')
    expect(dialog!.textContent).not.toContain('网络连接失败')
    expect(dialog!.querySelector('img')?.getAttribute('src')).toBe('https://example.com/cover.png')
    expect(searchButton!.disabled).toBe(false)
  })

  // 测试场景：未匹配到游戏或匹配游戏没有封面时，展示对应空状态，并禁用确认按钮。
  it.each([
    { matchedGame: null, message: '没有找到匹配的游戏' },
    {
      matchedGame: { provider: 'steamgriddb', providerGameId: '1314', name: '空洞骑士' },
      message: '暂无可用的竖版封面'
    }
  ])('shows the empty state: $message', async ({ matchedGame, message }) => {
    vi.mocked(searchCoverCandidates).mockResolvedValueOnce({ matchedGame, alternativeGames: [], candidates: [] })
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '空洞骑士' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(searchCoverCandidates).toHaveBeenCalledTimes(1)
    expect(dialog!.textContent).toContain(message)
    expect(dialog!.querySelector('img')).toBeNull()
    expect(dialog!.querySelector('[role="status"]')).toBeNull()
    const confirmButton = Array.from(dialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    expect(confirmButton).toBeDefined()
    expect(confirmButton!.disabled).toBe(true)
    expect(wrapper.emitted('select')).toBeUndefined()
  })

  it('emits cover when click submit button', async () => {
    vi.mocked(searchCoverCandidates).mockResolvedValueOnce({
      matchedGame: { provider: 'steamgriddb', providerGameId: '1314', name: '空洞骑士' },
      alternativeGames: [],
      candidates: [
        {
          provider: 'steamgriddb',
          providerGameId: '1314',
          assetId: 'cover-1',
          previewUrl: 'https://example.com/cover.png'
        }
      ]
    })

    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '空洞骑士' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    const coverButton = document.body.querySelector<HTMLButtonElement>('[aria-label="选择此封面"]')
    coverButton?.click()
    await nextTick()
    expect(coverButton?.getAttribute('aria-label')).toBe('已选择此封面')

    // 确定
    const coverCandidate: CoverCandidate = {
      assetId: 'cover-1',
      previewUrl: 'https://example.com/cover.png',
      provider: 'steamgriddb',
      providerGameId: '1314'
    }
    const confirmButton = Array.from(dialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    await flushPromises()
    expect(confirmButton).toBeDefined()
    expect(confirmButton!.disabled).toBe(false)
    confirmButton?.click()
    expect(wrapper.emitted('select')).toEqual([[coverCandidate]])
  })

  // 测试场景：选中游戏 A 的封面后关闭，再为游戏 B 打开时，不能确认或提交 A 的旧封面。
  it('clears the selected cover when reopened for another game', async () => {
    const oldCover: CoverCandidate = {
      provider: 'steamgriddb',
      providerGameId: 'game-a',
      assetId: 'cover-a',
      previewUrl: 'https://example.com/cover-a.png'
    }
    const newCover: CoverCandidate = {
      provider: 'steamgriddb',
      providerGameId: 'game-b',
      assetId: 'cover-b',
      previewUrl: 'https://example.com/cover-b.png'
    }
    vi.mocked(searchCoverCandidates)
      .mockResolvedValueOnce({
        matchedGame: { provider: 'steamgriddb', providerGameId: 'game-a', name: '游戏 A' },
        alternativeGames: [],
        candidates: [oldCover]
      })
      .mockResolvedValueOnce({
        matchedGame: { provider: 'steamgriddb', providerGameId: 'game-b', name: '游戏 B' },
        alternativeGames: [],
        candidates: [newCover]
      })

    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '游戏 A' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const firstDialog = document.body.querySelector('[role="dialog"]')
    expect(firstDialog).not.toBeNull()
    const coverButton = firstDialog!.querySelector<HTMLButtonElement>('[aria-label="选择此封面"]')
    expect(coverButton).not.toBeNull()
    coverButton!.click()
    await nextTick()
    const firstConfirmButton = Array.from(firstDialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    expect(firstConfirmButton).toBeDefined()
    expect(firstConfirmButton!.disabled).toBe(false)

    await wrapper.setProps({ open: false })
    await wrapper.setProps({ open: true, initialQuery: '游戏 B' })
    await flushPromises()

    const reopenedDialog = document.body.querySelector('[role="dialog"]')
    expect(reopenedDialog).not.toBeNull()
    expect(searchCoverCandidates).toHaveBeenCalledTimes(2)
    expect(searchCoverCandidates).toHaveBeenNthCalledWith(2, '游戏 B')
    expect(reopenedDialog!.querySelector('img')?.getAttribute('src')).toBe(newCover.previewUrl)
    expect(reopenedDialog!.querySelector('[aria-label="已选择此封面"]')).toBeNull()
    const confirmButton = Array.from(reopenedDialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    expect(confirmButton).toBeDefined()
    expect(confirmButton!.disabled).toBe(true)
    confirmButton!.click()
    await nextTick()
    expect(wrapper.emitted('select')).toBeUndefined()
  })

  // 测试场景：通过下拉框切换匹配游戏后，加载新游戏的封面并清空原有选择，重新选择前不能确认。
  it('loads covers and clears the selection when switching the matched game', async () => {
    const oldCover: CoverCandidate = {
      provider: 'steamgriddb',
      providerGameId: 'game-a',
      assetId: 'cover-a',
      previewUrl: 'https://example.com/cover-a.png'
    }
    const newCover: CoverCandidate = {
      provider: 'steamgriddb',
      providerGameId: 'game-b',
      assetId: 'cover-b',
      previewUrl: 'https://example.com/cover-b.png'
    }
    vi.mocked(searchCoverCandidates).mockResolvedValueOnce({
      matchedGame: { provider: 'steamgriddb', providerGameId: 'game-a', name: '游戏 A' },
      alternativeGames: [{ provider: 'steamgriddb', providerGameId: 'game-b', name: '游戏 B' }],
      candidates: [oldCover]
    })
    vi.mocked(listCoverCandidates).mockResolvedValueOnce([newCover])
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '游戏 A' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    const coverButton = dialog!.querySelector<HTMLButtonElement>('[aria-label="选择此封面"]')
    expect(coverButton).not.toBeNull()
    coverButton!.click()
    await nextTick()
    const confirmButton = Array.from(dialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    expect(confirmButton).toBeDefined()
    expect(confirmButton!.disabled).toBe(false)

    const gameSelect = dialog!.querySelector<HTMLButtonElement>('[role="combobox"]')
    expect(gameSelect).not.toBeNull()
    await new DOMWrapper(gameSelect!).trigger('keydown', { key: 'Enter' })
    const nextGameOption = Array.from(document.body.querySelectorAll<HTMLElement>('[role="option"]')).find(
      (option) => option.textContent?.trim() === '游戏 B'
    )
    expect(nextGameOption).toBeDefined()
    await new DOMWrapper(nextGameOption!).trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(listCoverCandidates).toHaveBeenCalledTimes(1)
    expect(listCoverCandidates).toHaveBeenCalledWith('steamgriddb', 'game-b')
    expect(gameSelect!.textContent?.trim()).toBe('游戏 B')
    expect(dialog!.querySelectorAll('img')).toHaveLength(1)
    expect(dialog!.querySelector('img')?.getAttribute('src')).toBe(newCover.previewUrl)
    expect(dialog!.querySelector('[aria-label="已选择此封面"]')).toBeNull()
    expect(confirmButton!.disabled).toBe(true)
    confirmButton!.click()
    expect(wrapper.emitted('select')).toBeUndefined()
  })

  it('ignores an older search response after a newer search completes', async () => {
    let resolveA!: (value: CoverSearchResult) => void
    const requestA = new Promise<CoverSearchResult>((resolve) => {
      resolveA = resolve
    })
    const resultA: CoverSearchResult = {
      matchedGame: { provider: 'steamgriddb', providerGameId: 'game-a', name: '游戏 A' },
      alternativeGames: [],
      candidates: [
        {
          provider: 'steamgriddb',
          providerGameId: 'game-a',
          assetId: 'cover-a',
          previewUrl: 'https://example.com/cover-a.png'
        }
      ]
    }
    const resultB: CoverSearchResult = {
      matchedGame: { provider: 'steamgriddb', providerGameId: 'game-b', name: '游戏 B' },
      alternativeGames: [],
      candidates: [
        {
          provider: 'steamgriddb',
          providerGameId: 'game-b',
          assetId: 'cover-b',
          previewUrl: 'https://example.com/cover-b.png'
        }
      ]
    }
    vi.mocked(searchCoverCandidates).mockReturnValueOnce(requestA).mockResolvedValueOnce(resultB)

    // 弹框
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '游戏 A' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()

    // 输入 / 按钮
    const input = dialog!.querySelector<HTMLInputElement>('input')
    expect(input).not.toBeNull()
    await new DOMWrapper(input!).setValue('游戏 B')
    const searchButton = dialog!.querySelector<HTMLButtonElement>('[type="submit"]')
    expect(searchButton).not.toBeNull()
    searchButton!.click()
    await flushPromises()

    // 结果
    const candidatesImages = document.querySelectorAll<HTMLImageElement>('img')
    expect(candidatesImages.length).toBe(1)
    expect(candidatesImages[0].getAttribute('src')).toBe('https://example.com/cover-b.png')

    resolveA(resultA)
    await flushPromises()
    const finalImages = document.querySelectorAll<HTMLImageElement>('img')
    expect(finalImages.length).toBe(1)
    expect(finalImages[0].getAttribute('src')).toBe('https://example.com/cover-b.png')
  })

  // 测试场景：B 搜索成功后，A 的旧请求才失败，不能显示旧错误或覆盖 B 的结果。
  it('ignores an older search error after a newer search completes', async () => {
    let rejectA!: (reason: Error) => void
    const requestA = new Promise<CoverSearchResult>((_resolve, reject) => {
      rejectA = reject
    })
    vi.mocked(searchCoverCandidates)
      .mockReturnValueOnce(requestA)
      .mockResolvedValueOnce({
        matchedGame: { provider: 'steamgriddb', providerGameId: 'game-b', name: '游戏 B' },
        alternativeGames: [],
        candidates: [
          {
            provider: 'steamgriddb',
            providerGameId: 'game-b',
            assetId: 'cover-b',
            previewUrl: 'https://example.com/cover-b.png'
          }
        ]
      })
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '游戏 A' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    const input = dialog!.querySelector<HTMLInputElement>('input')
    expect(input).not.toBeNull()
    await new DOMWrapper(input!).setValue('游戏 B')
    const searchButton = dialog!.querySelector<HTMLButtonElement>('[type="submit"]')
    expect(searchButton).not.toBeNull()
    searchButton!.click()
    await flushPromises()
    expect(searchCoverCandidates).toHaveBeenCalledTimes(2)
    expect(searchCoverCandidates).toHaveBeenNthCalledWith(1, '游戏 A')
    expect(searchCoverCandidates).toHaveBeenNthCalledWith(2, '游戏 B')
    expect(dialog!.querySelector('img')?.getAttribute('src')).toBe('https://example.com/cover-b.png')

    rejectA(new Error('游戏 A 搜索失败'))
    await flushPromises()

    expect(dialog!.textContent).not.toContain('游戏 A 搜索失败')
    expect(dialog!.querySelector('[role="combobox"]')?.textContent?.trim()).toBe('游戏 B')
    expect(dialog!.querySelectorAll('img')).toHaveLength(1)
    expect(dialog!.querySelector('img')?.getAttribute('src')).toBe('https://example.com/cover-b.png')
    expect(dialog!.querySelector('[role="status"]')).toBeNull()
    expect(searchButton!.disabled).toBe(false)
  })

  // 测试场景：切换匹配游戏失败后显示错误、恢复下拉框操作，并禁止提交切换前已选中的封面。
  it('restores controls and clears the selected cover when switching games fails', async () => {
    vi.mocked(searchCoverCandidates).mockResolvedValueOnce({
      matchedGame: { provider: 'steamgriddb', providerGameId: 'game-a', name: '游戏 A' },
      alternativeGames: [{ provider: 'steamgriddb', providerGameId: 'game-b', name: '游戏 B' }],
      candidates: [
        {
          provider: 'steamgriddb',
          providerGameId: 'game-a',
          assetId: 'cover-a',
          previewUrl: 'https://example.com/cover-a.png'
        }
      ]
    })
    vi.mocked(listCoverCandidates).mockRejectedValueOnce(new Error('加载游戏 B 封面失败'))
    const wrapper = mount(OnlineCoverDialog, {
      attachTo: document.body,
      props: { open: false, initialQuery: '游戏 A' }
    })
    await wrapper.setProps({ open: true })
    await flushPromises()

    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    const coverButton = dialog!.querySelector<HTMLButtonElement>('[aria-label="选择此封面"]')
    expect(coverButton).not.toBeNull()
    coverButton!.click()
    await nextTick()
    const confirmButton = Array.from(dialog!.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '使用此封面'
    )
    expect(confirmButton).toBeDefined()
    expect(confirmButton!.disabled).toBe(false)

    const gameSelect = dialog!.querySelector<HTMLButtonElement>('[role="combobox"]')
    expect(gameSelect).not.toBeNull()
    await new DOMWrapper(gameSelect!).trigger('keydown', { key: 'Enter' })
    const nextGameOption = Array.from(document.body.querySelectorAll<HTMLElement>('[role="option"]')).find(
      (option) => option.textContent?.trim() === '游戏 B'
    )
    expect(nextGameOption).toBeDefined()
    await new DOMWrapper(nextGameOption!).trigger('keydown', { key: 'Enter' })
    await flushPromises()

    expect(listCoverCandidates).toHaveBeenCalledTimes(1)
    expect(listCoverCandidates).toHaveBeenCalledWith('steamgriddb', 'game-b')
    expect(dialog!.textContent).toContain('加载游戏 B 封面失败')
    expect(gameSelect!.disabled).toBe(false)
    expect(gameSelect!.textContent?.trim()).toBe('游戏 A')
    expect(dialog!.querySelector('[role="status"]')).toBeNull()
    expect(dialog!.querySelector('[aria-label="已选择此封面"]')).toBeNull()
    expect(confirmButton!.disabled).toBe(true)
    confirmButton!.click()
    expect(wrapper.emitted('select')).toBeUndefined()
  })
})
