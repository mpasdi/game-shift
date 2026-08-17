import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import BaseSwitch from './BaseSwitch.vue'

describe('BaseSwitch', () => {
  it('测试初始状态', () => {
    const wrapper = mount(BaseSwitch, {
      props: {
        modelValue: false,
        accessibleLabel: '测试'
      },
      slots: {
        default: 'test'
      }
    })

    const switchController = wrapper.get('[role="switch"]')
    expect(switchController.attributes('aria-label')).toBe('测试')
    expect(switchController.attributes('aria-checked')).toBe('false')
    expect(switchController.text()).toContain('test')
  })

  it('emits next value when clicked', async () => {
    const wrapper = mount(BaseSwitch, {
      props: {
        modelValue: false,
        accessibleLabel: '测试'
      }
    })
    const switchController = wrapper.get('[role="switch"]')
    await switchController.trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([[true]])
  })

  it('test disabled', async () => {
    const wrapper = mount(BaseSwitch, {
      props: {
        modelValue: false,
        accessibleLabel: 'test',
        disabled: true
      }
    })
    const switchController = wrapper.get<HTMLButtonElement>('[role="switch"]')
    expect(switchController.element.disabled).toBe(true)
    await switchController.trigger('click')
    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('test loading', async () => {
    const wrapper = mount(BaseSwitch, {
      props: {
        modelValue: false,
        accessibleLabel: 'test',
        loading: true
      }
    })

    const switchController = wrapper.get<HTMLButtonElement>('[role="switch"]')
    expect(switchController.element.disabled).toBe(false)
    expect(switchController.attributes('aria-disabled')).toBe('true')
    expect(switchController.attributes('aria-busy')).toBe('true')
    await switchController.trigger('click')
    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })
})
