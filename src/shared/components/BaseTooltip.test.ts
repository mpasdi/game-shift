import { enableAutoUnmount, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import BaseTooltip from './BaseTooltip.vue'
import BaseTooltipProvider from './BaseTooltipProvider.vue'
import { defineComponent, nextTick } from 'vue'

enableAutoUnmount(afterEach)

const testComponent = defineComponent({
  components: {
    BaseTooltip,
    BaseTooltipProvider
  },
  props: {
    disabled: {
      type: Boolean,
      default: false
    }
  },
  template: `
      <BaseTooltipProvider>
        <BaseTooltip :disabled="disabled">
          <template #trigger>
            <button type="button">查看原因</button>
          </template>

          无法识别游戏入口
        </BaseTooltip>
      </BaseTooltipProvider>
    `
})

describe('BaseTooltip and BaseTooltipProvider', () => {
  it('shows its content when the trigger receivers is focus', async () => {
    const wrapper = mount(testComponent, {
      attachTo: document.body
    })
    const triggerBtn = wrapper.get('button')
    await triggerBtn.trigger('focus')
    await vi.waitFor(() => {
      const tooltipContent = document.body.querySelector('[role="tooltip"]')
      expect(tooltipContent).not.toBeNull()
      expect(tooltipContent?.textContent).toContain('无法识别游戏入口')
    })
  })

  it('dont show its content when disabled', async () => {
    const wrapper = mount(testComponent, {
      attachTo: document.body,
      props: {
        disabled: true
      }
    })

    await wrapper.trigger('focus')
    await nextTick()
    const tooltipContent = document.body.querySelector('[role="tooltip"]')
    expect(tooltipContent).toBeNull()
  })
})
