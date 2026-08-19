import { nextTick } from 'vue'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import BaseModal from './BaseModal.vue'

/**
 *   - DialogPortal：把弹框挂到 document.body，避免被父容器的 overflow、层级等限制。
 *   - attachTo: document.body：让被测组件挂载在真实的测试 DOM 中。
 *   - document.body.querySelector()：因为弹框内容已不在 wrapper 内，所以从页面根部查找。
 *   - role="dialog"：Reka 生成的标准弹框语义。
 *   - enableAutoUnmount(afterEach)：每个测试结束后自动卸载组件，避免上一个弹框污染下一个测试。
 */

enableAutoUnmount(afterEach)

describe('Modal', () => {
  it('render its content when open', async () => {
    mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        title: 'modal title'
      },
      slots: {
        default: 'modal content'
      }
    })
    await nextTick()

    const modal = document.body.querySelector('[role="dialog"]')
    expect(modal).not.toBeNull()
    expect(modal?.textContent).toContain('modal title')
    expect(modal?.textContent).toContain('modal content')
  })

  it('emits close when click the close button is clicked', async () => {
    const wrapper = mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        title: 'modal title'
      }
    })
    await nextTick()

    const closeButton = document.body.querySelector<HTMLButtonElement>('[aria-label="关闭"]')
    expect(closeButton).not.toBeNull()
    closeButton?.click()
    await nextTick()
    expect(wrapper.emitted('close')).toEqual([[]])
  })

  it('prevent closing when close is disabled', async () => {
    const wrapper = mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        closeDisabled: true,
        title: 'modal title'
      }
    })
    await nextTick()

    const closeButton = document.body.querySelector<HTMLButtonElement>('[aria-label="关闭"]')
    expect(closeButton).not.toBeNull()
    expect(closeButton?.disabled).toBe(true)

    closeButton?.click()
    await nextTick()
    expect(wrapper.emitted('close')).toBeUndefined()
  })

  /**
   *   - KeyboardEvent('keydown')：创建一次键盘按下事件。
   *   - key: 'Escape'：表示按下 Escape。
   *   - bubbles: true：允许事件沿 DOM 冒泡。
   *   - cancelable: true：允许组件调用 preventDefault() 阻止关闭。
   *   - 在 document 上触发：Reka 的弹框键盘监听位于文档层，而不是某个具体按钮。
   */
  it('test close modal when esc key is pressed', async () => {
    const wrapper = mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        title: 'modal title'
      }
    })

    await nextTick()
    document.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Escape',
        bubbles: true,
        cancelable: true
      })
    )
    await nextTick()
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('prevent Escape from closing when close is disabled ', async () => {
    const wrapper = mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        title: 'modal title',
        closeDisabled: true
      }
    })

    await nextTick()
    document.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Escape',
        bubbles: true,
        cancelable: true
      })
    )

    await nextTick()
    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('close then modal when click outside', async () => {
    const wrapper = mount(BaseModal, {
      attachTo: document.body,
      props: {
        open: true,
        title: 'modal title',
        closeOnBackdrop: true
      }
    })
    await nextTick()
    // 等待 Reka 注册弹框外部的 pointerdown 监听
    await new Promise((resolve) => setTimeout(resolve, 0))

    const backdrop = document.body.querySelector('.modal-backdrop')
    backdrop?.dispatchEvent(
      new Event('pointerdown', {
        bubbles: true,
        cancelable: true
      })
    )
    await nextTick()
    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
