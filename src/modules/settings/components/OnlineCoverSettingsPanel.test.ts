import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import OnlineCoverSettingsPanel from './OnlineCoverSettingsPanel.vue'
import {
  deleteSteamGridDbApiKey,
  getOnlineCoverSettings,
  saveSteamGridDbApiKey,
  setOnlineCoversEnabled,
  testSteamGridDbConnection,
  type OnlineCoverSettings
} from '../api'

vi.mock('../api', () => ({
  getOnlineCoverSettings: vi.fn(),
  setOnlineCoversEnabled: vi.fn(),
  saveSteamGridDbApiKey: vi.fn(),
  deleteSteamGridDbApiKey: vi.fn(),
  testSteamGridDbConnection: vi.fn()
}))
const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }))
vi.mock('../../../shared/composables/useToast', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../../shared/composables/useToast')>()),
  useToast: () => toast
}))

enableAutoUnmount(afterEach)
beforeEach(() => vi.resetAllMocks())

const disabled: OnlineCoverSettings = { enabled: false, hasApiKey: false, state: 'disabled' }
const missing: OnlineCoverSettings = { enabled: true, hasApiKey: false, state: 'missingApiKey' }
const ready: OnlineCoverSettings = { enabled: true, hasApiKey: true, apiKeyHint: '****1234', state: 'ready' }

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function button(label: string, root: ParentNode = document.body) {
  const element = Array.from(root.querySelectorAll('button')).find((item) => item.textContent?.trim() === label)
  expect(element, `按钮：${label}`).toBeDefined()
  return element!
}

async function render(settings = ready) {
  vi.mocked(getOnlineCoverSettings).mockResolvedValueOnce(settings)
  const wrapper = mount(OnlineCoverSettingsPanel, { attachTo: document.body })
  await flushPromises()
  return wrapper
}

describe('OnlineCoverSettingsPanel', () => {
  // 测试场景：读取完成前禁用开关；读取成功后展示保存的配置。
  it('disables the switch while loading and displays saved settings', async () => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(getOnlineCoverSettings).mockReturnValueOnce(pending.promise)
    const wrapper = mount(OnlineCoverSettingsPanel, { attachTo: document.body })
    await nextTick()
    expect(wrapper.text()).toContain('正在读取联网封面设置')
    expect(wrapper.get<HTMLButtonElement>('[role="switch"]').element.disabled).toBe(true)
    pending.resolve(ready)
    await flushPromises()
    expect(getOnlineCoverSettings).toHaveBeenCalledTimes(1)
    expect(wrapper.text()).toContain('已启用且已配置')
    expect(wrapper.text()).toContain('****1234')
    expect(wrapper.get('[role="switch"]').attributes('aria-checked')).toBe('true')
  })

  // 测试场景：读取失败时显示原因，重新读取成功后移除错误并恢复操作。
  it('retries loading settings after a failure', async () => {
    vi.mocked(getOnlineCoverSettings).mockRejectedValueOnce(new Error('读取失败')).mockResolvedValueOnce(disabled)
    const wrapper = mount(OnlineCoverSettingsPanel, { attachTo: document.body })
    await flushPromises()
    expect(wrapper.text()).toContain('读取失败')
    button('重新读取').click()
    await flushPromises()
    expect(getOnlineCoverSettings).toHaveBeenCalledTimes(2)
    expect(wrapper.text()).not.toContain('读取失败')
    expect(wrapper.text()).toContain('未启用')
    expect(wrapper.get<HTMLButtonElement>('[role="switch"]').element.disabled).toBe(false)
  })

  // 测试场景：开启和关闭均保存到后端，等待期间禁止重复操作，完成后显示新配置。
  it.each([true, false])('persists the enabled state: %s', async (enabled) => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(setOnlineCoversEnabled).mockReturnValueOnce(pending.promise)
    const wrapper = await render(enabled ? disabled : missing)
    const toggle = wrapper.get<HTMLButtonElement>('[role="switch"]')
    await toggle.trigger('click')
    expect(toggle.attributes('aria-checked')).toBe(String(enabled))
    expect(toggle.attributes('aria-disabled')).toBe('true')
    expect(toggle.attributes('aria-busy')).toBe('true')
    toggle.element.click()
    expect(setOnlineCoversEnabled).toHaveBeenCalledTimes(1)
    expect(setOnlineCoversEnabled).toHaveBeenCalledWith(enabled)
    pending.resolve(enabled ? missing : disabled)
    await flushPromises()
    expect(toggle.element.disabled).toBe(false)
    expect(wrapper.find('input').exists()).toBe(enabled)
    expect(toast.success).toHaveBeenCalledWith({ title: enabled ? '联网封面已启用' : '联网封面已关闭' })
  })

  // 测试场景：保存开关失败时撤销乐观状态，保留原配置并提示错误。
  it.each([true, false])('rolls back a failed toggle from enabled=%s', async (enabled) => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(setOnlineCoversEnabled).mockReturnValueOnce(pending.promise)
    const wrapper = await render(enabled ? missing : disabled)
    const toggle = wrapper.get<HTMLButtonElement>('[role="switch"]')
    await toggle.trigger('click')
    expect(toggle.attributes('aria-checked')).toBe(String(!enabled))
    pending.reject(new Error('保存设置失败'))
    await flushPromises()
    expect(toggle.attributes('aria-checked')).toBe(String(enabled))
    expect(toggle.element.disabled).toBe(false)
    expect(toast.error).toHaveBeenCalledWith({ title: '更新联网封面设置失败', description: '保存设置失败' })
  })

  // 测试场景：空白 Key 不提交；有效 Key 去除首尾空格后保存，等待期间阻止重复提交。
  it('validates and saves a trimmed key without duplicate submissions', async () => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(saveSteamGridDbApiKey).mockReturnValueOnce(pending.promise)
    const wrapper = await render(missing)
    await wrapper.get('input').setValue('   ')
    expect(button('验证并保存').disabled).toBe(true)
    await wrapper.get('form').trigger('submit')
    expect(saveSteamGridDbApiKey).not.toHaveBeenCalled()
    await wrapper.get('input').setValue('  new-key  ')
    await wrapper.get('form').trigger('submit')
    expect(button('验证并保存').disabled).toBe(true)
    expect(wrapper.get<HTMLButtonElement>('[role="switch"]').element.disabled).toBe(true)
    await wrapper.get('form').trigger('submit')
    expect(saveSteamGridDbApiKey).toHaveBeenCalledTimes(1)
    expect(saveSteamGridDbApiKey).toHaveBeenCalledWith('new-key')
    pending.resolve(ready)
    await flushPromises()
    expect(wrapper.find('input').exists()).toBe(false)
    expect(wrapper.text()).toContain('****1234')
    expect(toast.success).toHaveBeenCalledWith({ title: 'SteamGridDB API Key 验证成功并已保存' })
  })

  // 测试场景：更换 Key 失败时保留输入供重试，重试成功后展示新 Key 的摘要。
  it('preserves the replacement key on failure and allows retry', async () => {
    vi.mocked(saveSteamGridDbApiKey)
      .mockRejectedValueOnce(new Error('Key 无效'))
      .mockResolvedValueOnce({ ...ready, apiKeyHint: '****5678' })
    const wrapper = await render()
    button('更换').click()
    await nextTick()
    await wrapper.get('input').setValue('replacement-key')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.get<HTMLInputElement>('input').element.value).toBe('replacement-key')
    expect(button('验证并保存').disabled).toBe(false)
    expect(toast.error).toHaveBeenCalledWith({ title: '保存失败', description: 'Key 无效' })
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(saveSteamGridDbApiKey).toHaveBeenCalledTimes(2)
    expect(wrapper.text()).toContain('****5678')
    expect(wrapper.find('input').exists()).toBe(false)
  })

  // 测试场景：取消更换不写入后端，重新更换时不保留上次输入。
  it('discards an unsaved replacement when cancelled', async () => {
    const wrapper = await render()
    button('更换').click()
    await nextTick()
    await wrapper.get('input').setValue('discarded-key')
    button('取消').click()
    await nextTick()
    expect(wrapper.text()).toContain('****1234')
    expect(saveSteamGridDbApiKey).not.toHaveBeenCalled()
    button('更换').click()
    await nextTick()
    expect(wrapper.get<HTMLInputElement>('input').element.value).toBe('')
  })

  // 测试场景：重新验证期间锁定操作，成功后更新配置状态。
  it('revalidates the key and prevents concurrent operations', async () => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(testSteamGridDbConnection).mockReturnValueOnce(pending.promise)
    const wrapper = await render({ ...ready, state: 'invalidApiKey' })
    button('重新验证').click()
    await nextTick()
    for (const label of ['重新验证', '更换', '删除']) expect(button(label).disabled).toBe(true)
    button('重新验证').click()
    expect(testSteamGridDbConnection).toHaveBeenCalledTimes(1)
    pending.resolve(ready)
    await flushPromises()
    expect(wrapper.text()).toContain('已启用且已配置')
    expect(button('重新验证').disabled).toBe(false)
    expect(toast.success).toHaveBeenCalledWith({
      title: 'SteamGridDB 连接成功',
      description: '当前 API Key 可以正常使用'
    })
  })

  // 测试场景：连接失败后刷新 Key 状态；刷新也失败时保留原配置，始终提示原始连接错误。
  it.each([true, false])('handles a failed connection with refresh success=%s', async (refreshSucceeds) => {
    const wrapper = await render()
    if (refreshSucceeds) vi.mocked(getOnlineCoverSettings).mockResolvedValueOnce({ ...ready, state: 'invalidApiKey' })
    else vi.mocked(getOnlineCoverSettings).mockRejectedValueOnce(new Error('刷新失败'))
    vi.mocked(testSteamGridDbConnection).mockRejectedValueOnce(new Error('连接失败'))
    button('重新验证').click()
    await flushPromises()
    expect(getOnlineCoverSettings).toHaveBeenCalledTimes(2)
    expect(wrapper.text()).toContain(refreshSucceeds ? '配置异常' : '已启用且已配置')
    expect(button('重新验证').disabled).toBe(false)
    expect(toast.error).toHaveBeenCalledWith({ title: 'SteamGridDB 连接失败', description: '连接失败' })
  })

  // 测试场景：删除需要确认，取消时不调用删除接口。
  it('does not delete the key when confirmation is cancelled', async () => {
    const wrapper = await render()
    button('删除').click()
    await nextTick()
    const dialog = document.body.querySelector('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(deleteSteamGridDbApiKey).not.toHaveBeenCalled()
    button('取消', dialog!).click()
    await flushPromises()
    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    expect(wrapper.text()).toContain('****1234')
    expect(deleteSteamGridDbApiKey).not.toHaveBeenCalled()
  })

  // 测试场景：删除中禁止重复确认或取消；成功关闭弹窗，清除 Key 但保持联网开关开启。
  it('deletes the key once and keeps online covers enabled', async () => {
    const pending = deferred<OnlineCoverSettings>()
    vi.mocked(deleteSteamGridDbApiKey).mockReturnValueOnce(pending.promise)
    const wrapper = await render()
    button('删除').click()
    await nextTick()
    button('确认删除').click()
    await nextTick()
    expect(button('确认删除').disabled).toBe(true)
    expect(button('取消').disabled).toBe(true)
    button('确认删除').click()
    button('取消').click()
    expect(deleteSteamGridDbApiKey).toHaveBeenCalledTimes(1)
    expect(document.body.querySelector('[role="dialog"]')).not.toBeNull()
    pending.resolve(missing)
    await flushPromises()
    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    expect(wrapper.text()).toContain('已启用但未配置')
    expect(wrapper.text()).not.toContain('****1234')
    expect(wrapper.get('[role="switch"]').attributes('aria-checked')).toBe('true')
    expect(toast.success).toHaveBeenCalledWith({ title: 'SteamGridDB API Key 已删除' })
  })

  // 测试场景：删除失败保留确认弹窗和原 Key，恢复按钮供重试。
  it('keeps the key and confirmation after deletion fails and allows retry', async () => {
    vi.mocked(deleteSteamGridDbApiKey).mockRejectedValueOnce(new Error('删除失败')).mockResolvedValueOnce(missing)
    const wrapper = await render()
    button('删除').click()
    await nextTick()
    button('确认删除').click()
    await flushPromises()
    expect(document.body.querySelector('[role="dialog"]')).not.toBeNull()
    expect(wrapper.text()).toContain('****1234')
    expect(button('确认删除').disabled).toBe(false)
    expect(button('取消').disabled).toBe(false)
    expect(toast.error).toHaveBeenCalledWith({ title: '删除 API Key 失败', description: '删除失败' })
    button('确认删除').click()
    await flushPromises()
    expect(deleteSteamGridDbApiKey).toHaveBeenCalledTimes(2)
    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    expect(wrapper.text()).toContain('已启用但未配置')
  })
})
