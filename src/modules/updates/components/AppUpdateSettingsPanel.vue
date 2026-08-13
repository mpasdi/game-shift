<script setup lang="ts">
  import { computed, onMounted, ref } from 'vue'
  import { storeToRefs } from 'pinia'
  import { CheckCircle2, Download, RefreshCw, TriangleAlert } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseSwitch from '../../../shared/components/BaseSwitch.vue'
  import { getErrorMessage, useToast } from '../../../shared/composables/useToast'
  import { useAppUpdaterStore } from '../stores/appUpdater'

  const updater = useAppUpdaterStore()
  const toast = useToast()
  const {
    settings,
    settingsError,
    autoCheckEnabled,
    availableUpdate,
    isChecking,
    isInstalling,
    lastCheckResult,
    lastError
  } = storeToRefs(updater)
  const isSettingsLoading = ref(false)
  const isToggling = ref(false)
  const pendingAutoCheckEnabled = ref<boolean | null>(null)
  const displayedAutoCheckEnabled = computed(() => pendingAutoCheckEnabled.value ?? autoCheckEnabled.value)

  const status = computed(() => {
    if (availableUpdate.value) {
      return {
        type: 'available',
        label: `发现新版本 v${availableUpdate.value.version}`,
        description: '可以查看更新内容并选择是否安装。'
      }
    }

    if (lastCheckResult.value === 'upToDate') {
      return { type: 'ready', label: '当前已是最新版本', description: '暂时没有可用的新版本。' }
    }

    if (lastCheckResult.value === 'failed') {
      return { type: 'error', label: '检查更新失败', description: lastError.value || '请稍后重新检查。' }
    }

    if (settingsError.value && !settings.value) {
      return { type: 'error', label: '更新设置读取失败', description: settingsError.value }
    }

    return {
      type: 'idle',
      label: displayedAutoCheckEnabled.value ? '自动检查已开启' : '自动检查已关闭',
      description: displayedAutoCheckEnabled.value
        ? '应用和游戏库加载完成后会检查一次新版本。'
        : '仍可随时手动检查新版本。'
    }
  })

  async function loadSettings() {
    isSettingsLoading.value = true
    try {
      await updater.loadSettings()
    } catch {
      // 读取错误直接展示在当前面板中。
    } finally {
      isSettingsLoading.value = false
    }
  }

  async function toggleAutoCheck(enabled: boolean) {
    if (!settings.value || isToggling.value) return

    pendingAutoCheckEnabled.value = enabled
    isToggling.value = true
    try {
      const next = await updater.setAutoCheckEnabled(enabled)
      toast.success({ title: next.autoCheckEnabled ? '自动检查更新已开启' : '自动检查更新已关闭' })
    } catch (error) {
      toast.error({ title: '更新设置保存失败', description: getErrorMessage(error) })
    } finally {
      pendingAutoCheckEnabled.value = null
      isToggling.value = false
    }
  }

  onMounted(() => {
    void loadSettings()
  })
</script>

<template>
  <article class="app-update-panel">
    <div class="app-update-panel__heading">
      <div class="app-update-panel__title">
        <RefreshCw :size="16" />
        <div>
          <h2>应用更新</h2>
          <p>获取 Game Shift 的新版本</p>
        </div>
      </div>

      <div class="app-update-panel__controls">
        <BaseSwitch
          :model-value="displayedAutoCheckEnabled"
          :accessible-label="displayedAutoCheckEnabled ? '关闭自动检查更新' : '开启自动检查更新'"
          :disabled="!settings || isSettingsLoading"
          :loading="isToggling"
          @update:model-value="toggleAutoCheck"
        >
          {{ displayedAutoCheckEnabled ? '自动检查已开启' : '自动检查已关闭' }}
        </BaseSwitch>

        <BaseButton :loading="isChecking" :disabled="isInstalling" size="sm" @click="updater.checkForUpdates()">
          检查更新
        </BaseButton>
      </div>
    </div>

    <div class="app-update-panel__body">
      <div class="app-update-status" :class="`app-update-status--${status.type}`">
        <Download v-if="status.type === 'available'" :size="18" />
        <TriangleAlert v-else-if="status.type === 'error'" :size="18" />
        <CheckCircle2 v-else :size="18" />
        <div>
          <strong>{{ status.label }}</strong>
          <p>{{ status.description }}</p>
        </div>
      </div>

      <BaseButton v-if="availableUpdate" variant="primary" size="sm" @click="updater.openUpdateDialog">
        查看更新
      </BaseButton>
    </div>
  </article>
</template>

<style scoped>
  .app-update-panel {
    min-width: 0;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.24);
  }

  .app-update-panel__heading,
  .app-update-panel__title,
  .app-update-panel__controls,
  .app-update-panel__body,
  .app-update-status {
    display: flex;
    align-items: center;
  }

  .app-update-panel__heading {
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
    padding: 12px 16px;
  }

  .app-update-panel__title,
  .app-update-status {
    gap: 8px;
  }

  .app-update-panel__title > svg,
  .app-update-status {
    color: var(--accent-strong);
  }

  .app-update-panel__title > svg {
    margin-top: 1px;
  }

  .app-update-panel__controls {
    gap: 10px;
  }

  .app-update-panel h2,
  .app-update-panel p {
    margin: 0;
  }

  .app-update-panel h2 {
    font-size: var(--font-size-md);
    line-height: 1.2;
  }

  .app-update-panel__title p {
    margin-top: 4px;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
  }

  .app-update-panel__body {
    justify-content: space-between;
    gap: 16px;
    min-height: 76px;
    padding: 14px 16px;
  }

  .app-update-status {
    align-items: flex-start;
  }

  .app-update-status--error {
    color: #fca5a5;
  }

  .app-update-status strong {
    color: var(--text);
    font-size: var(--font-size-sm);
  }

  .app-update-status p {
    margin-top: 4px;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
  }

  @media (max-width: 720px) {
    .app-update-panel__heading,
    .app-update-panel__controls,
    .app-update-panel__body {
      align-items: stretch;
      flex-direction: column;
    }

    .app-update-panel__heading {
      gap: 12px;
    }

    .base-switch {
      align-self: flex-start;
    }
  }
</style>
