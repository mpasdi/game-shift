import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import BaseButton from './BaseButton.vue'

/**
 *   - mount()：把 Vue 组件挂载到 happy-dom 提供的模拟页面中。
 *   - describe()：定义一组与 BaseButton 有关的测试。
 *   - it()：定义一个具体行为。
 *   - slots.default：向按钮默认插槽传入“保存”。
 *   - wrapper：挂载后的组件包装器，可以查询元素和模拟交互。
 *   - get('button')：查找按钮；找不到会直接让测试失败。
 *   - expect(...).toBe(...)：断言实际结果必须严格等于预期结果
 *   - vi.fn()：创建一个 mock 函数，会记录自己被调用了多少次。
 *   - attrs.onClick：模拟父组件写下的 @click="onClick"。
 *   - trigger('click')：模拟用户点击按钮。
 *   - await：Vue 的事件和视图更新可能是异步的，组件测试中应等待交互完成。
 *   - toHaveBeenCalledTimes(1)：断言回调恰好执行一次。
 *   - get<HTMLButtonElement>()：告诉 TypeScript，这个元素是原生按钮。
 *   - button.element：取得 happy-dom 中的原生 DOM 元素。
 *   - .not.toHaveBeenCalled()：断言回调一次都没有执行。
 */

describe('BaseButton', () => {
  it('renders the default slot content', () => {
    const wrapper = mount(BaseButton, {
      slots: {
        default: '保存'
      }
    })

    expect(wrapper.get('button').text()).toBe('保存')
  })

  it('按钮点击事件', async () => {
    const onClick = vi.fn()
    const wrapper = mount(BaseButton, {
      slots: {
        default: '保存'
      },
      attrs: {
        onClick
      }
    })

    await wrapper.get('button').trigger('click')
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('prevents clicks when disabled', async () => {
    const onClick = vi.fn()
    const wrapper = mount(BaseButton, {
      slots: {
        default: '保存'
      },
      props: {
        disabled: true
      },
      attrs: {
        onClick
      }
    })

    const button = wrapper.get<HTMLButtonElement>('button')
    expect(button.element.disabled).toBe(true)
    await button.trigger('click')
    expect(onClick).not.toHaveBeenCalled()
  })

  it('exposes loading state and prevents clicks ', async () => {
    const onClick = vi.fn()
    const wrapper = mount(BaseButton, {
      attrs: {
        onClick
      },
      props: {
        loading: true
      },
      slots: {
        default: '保存'
      }
    })
    const button = wrapper.get<HTMLButtonElement>('button')

    expect(button.element.disabled).toBe(true)
    expect(button.attributes('aria-busy')).toBe('true')
    await button.trigger('click')
    expect(onClick).not.toHaveBeenCalled()
  })
})
