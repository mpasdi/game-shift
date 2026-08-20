import { mount, enableAutoUnmount } from '@vue/test-utils'
import { describe, expect, it, afterEach, vi } from 'vitest'
import BaseSelect from './BaseSelect.vue'
import { nextTick } from 'vue'

enableAutoUnmount(afterEach)
const baseProps = {
  modelValue: 'option1',
  options: [
    { label: 'label 1', value: 'option1' },
    { label: 'label 2', value: 'option2' }
  ],
  accessibleLabel: 'test'
}

describe('BaseSelect', () => {
  it('display the selected option', () => {
    const wrapper = mount(BaseSelect, {
      props: {
        modelValue: 'option1',
        options: [
          { label: 'label 1', value: 'option1' },
          { label: 'label 2', value: 'option2' }
        ],
        accessibleLabel: 'test'
      }
    })

    expect(wrapper.text()).toContain('label 1')
  })

  it('render options when open', async () => {
    const wrapper = mount(BaseSelect, {
      props: {
        modelValue: 'option1',
        options: [
          { label: 'label 1', value: 'option1' },
          { label: 'label 2', value: 'option2' }
        ],
        accessibleLabel: 'test'
      }
    })

    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    await selectButton.trigger('keydown', {
      key: 'Enter'
    })
    const selectPullDown = document.body.querySelector('[role="listbox"]')
    expect(selectPullDown).not.toBeNull()
    expect(selectPullDown?.textContent).toContain('label 1')
    expect(selectPullDown?.textContent).toContain('label 2')
  })

  it('emits the selected value', async () => {
    const wrapper = mount(BaseSelect, {
      props: { ...baseProps }
    })

    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    await selectButton.trigger('keydown', {
      key: 'Enter'
    })
    await nextTick()

    const options = document.body.querySelectorAll<HTMLElement>('[role="option"]')
    expect(options).toHaveLength(2)

    const secondOption = options[1]
    secondOption.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
        cancelable: true
      })
    )
    // 因为reka 内部实现的原因，单纯nextTick()无法保证事件处理完成，所以这里使用setTimeout来确保事件处理完成
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(wrapper.emitted('update:modelValue')).toEqual([['option2']])
  })

  it('prevent interaction when disabled', async () => {
    const wrapper = mount(BaseSelect, {
      props: { ...baseProps, disabled: true }
    })

    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    expect(selectButton.element.disabled).toBe(true)

    await selectButton.trigger('keydown', { key: 'Enter' })
    expect(document.body.querySelector('[role="listbox"]')).toBeNull()
  })

  it('expose loading state and disables interaction ', async () => {
    const wrapper = mount(BaseSelect, {
      props: { ...baseProps, loading: true }
    })

    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    expect(selectButton.element.disabled).toBe(true)
    expect(selectButton.attributes('aria-busy')).toBe('true')
    expect(wrapper.find('.base-select__spinner').exists()).toBe(true)
    await selectButton.trigger('keydown', { key: 'Enter' })
    expect(document.body.querySelector('[role="listbox"]')).toBeNull()
  })

  it('closes the dropdown when Escape is pressed', async () => {
    const wrapper = mount(BaseSelect, {
      props: { ...baseProps }
    })

    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    await selectButton.trigger('keydown', { key: 'Enter' })
    await nextTick()
    const selectPullDown = document.body.querySelector('[role="listbox"]')
    expect(selectPullDown).not.toBeNull()
    document.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Escape',
        bubbles: true,
        cancelable: true
      })
    )
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(selectButton.attributes('aria-expanded')).toBe('false')
    expect(document.body.querySelector('[role="listbox"]')).toBeNull()
  })

  it('test option can be selected when arrow key be keydown', async () => {
    const wrapper = mount(BaseSelect, {
      props: { ...baseProps }
    })
    const selectButton = wrapper.get<HTMLButtonElement>('[role="combobox"]')
    await selectButton.trigger('keydown', { key: 'Enter' })
    const options = document.body.querySelectorAll('[role="option"]')
    expect(options).toHaveLength(2)

    await vi.waitFor(() => {
      expect(document.activeElement).toBe(options[0])
    })

    options[0]?.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'ArrowDown',
        bubbles: true,
        cancelable: true
      })
    )
    await new Promise((resolve) => setTimeout(resolve, 0))

    await vi.waitFor(() => {
      expect(document.activeElement).toBe(options[1])
    })

    options[1].dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Enter'
      })
    )
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(wrapper.emitted('update:modelValue')).toEqual([['option2']])
  })
})
