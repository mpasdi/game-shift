import { createSharedComposable, useEventListener } from '@vueuse/core'
import { onActivated, onMounted, shallowRef, triggerRef } from 'vue'
import { formatLastPlayTime } from '../utils/formatLastPlayTime'

// 所有卡片与列表共用事件监听，最后一个使用者卸载时自动清理。
const useRelativeTimeFormatter = createSharedComposable(() => {
  const formatter = shallowRef(formatLastPlayTime)
  const refresh = () => {
    // 使使用者重新计算，格式化时仍读取实际系统时间。
    triggerRef(formatter)
  }

  useEventListener(window, ['focus', 'pageshow'], refresh)
  useEventListener(document, 'visibilitychange', () => {
    if (document.visibilityState === 'visible') refresh()
  })

  return { formatter, refresh }
})

export function useLastPlayTime() {
  const { formatter, refresh } = useRelativeTimeFormatter()
  onMounted(refresh)
  onActivated(refresh)

  return formatter
}
