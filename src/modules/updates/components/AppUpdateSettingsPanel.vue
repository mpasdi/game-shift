<script setup lang="ts">
  import { computed } from 'vue'
  import { storeToRefs } from 'pinia'
  import { CheckCircle2, Download, RefreshCw, TriangleAlert } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import { useAppUpdaterStore } from '../stores/appUpdater'

  const updater = useAppUpdaterStore()
  const { availableUpdate, isChecking, isInstalling, lastCheckResult, lastError } = storeToRefs(updater)

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

    return { type: 'idle', label: '手动检查更新', description: '检查 GitHub Releases 是否有可用的新版本。' }
  })
</script>

<template>
  <article class="app-update-panel">
    <div class="app-update-panel__heading">
      <div>
        <RefreshCw :size="16" />
        <h2>应用更新</h2>
      </div>
      <BaseButton :loading="isChecking" :disabled="isInstalling" size="sm" @click="updater.checkForUpdates">
        检查更新
      </BaseButton>
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
  .app-update-panel__heading > div,
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

  .app-update-panel__heading > div,
  .app-update-status {
    gap: 8px;
  }

  .app-update-panel__heading svg,
  .app-update-status {
    color: var(--accent-strong);
  }

  .app-update-panel h2,
  .app-update-panel p {
    margin: 0;
  }

  .app-update-panel h2 {
    font-size: var(--font-size-md);
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
    .app-update-panel__body {
      align-items: stretch;
      flex-direction: column;
    }
  }
</style>
