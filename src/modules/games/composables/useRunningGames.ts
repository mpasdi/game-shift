import { onMounted, onUnmounted } from 'vue'
import { useGamesStore } from '../stores/games'

// 只在桌面游戏库存活时同步；串行请求避免响应乱序和慢请求积压。
export function useRunningGames() {
  const store = useGamesStore()
  let stopped = false
  let timer: ReturnType<typeof setTimeout> | undefined
  async function poll() {
    await store.refreshRunningGames()
    if (!stopped) timer = setTimeout(() => void poll(), 1000)
  }
  onMounted(() => {
    if ('__TAURI_INTERNALS__' in window) void poll()
  })
  onUnmounted(() => {
    stopped = true
    clearTimeout(timer)
    ++store.runtimeRevision
  })
}
