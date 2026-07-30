<script setup lang="ts">
  import { computed } from 'vue'
  import { storeToRefs } from 'pinia'
  import { Download, RefreshCw, Rocket, TriangleAlert } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import { useAppUpdaterStore } from '../stores/appUpdater'

  const updater = useAppUpdaterStore()
  const { availableUpdate, isDialogOpen, phase, lastError, downloadProgress, isInstalling } = storeToRefs(updater)

  const formattedDate = computed(() => {
    const value = availableUpdate.value?.date
    if (!value) return '未提供'

    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value

    return new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit'
    }).format(date)
  })

  const statusText = computed(() => {
    if (phase.value === 'downloading') {
      return downloadProgress.value === null ? '正在下载更新...' : `正在下载更新 ${downloadProgress.value}%`
    }
    if (phase.value === 'installing') return '下载完成，正在安装...'
    if (phase.value === 'restarting') return '安装完成，正在重启应用...'
    if (phase.value === 'failed') return '更新未能完成'
    return null
  })
</script>

<template>
  <BaseModal
    :open="isDialogOpen && Boolean(availableUpdate)"
    title="发现新版本"
    size="md"
    :close-on-backdrop="!isInstalling"
    :close-disabled="isInstalling"
    @close="updater.dismissUpdate"
  >
    <div v-if="availableUpdate" class="app-update-dialog">
      <div class="app-update-dialog__hero">
        <span class="app-update-dialog__icon"><Rocket :size="26" /></span>
        <div>
          <p class="app-update-dialog__eyebrow">Game Shift 更新</p>
          <h3>v{{ availableUpdate.version }}</h3>
          <p>当前版本 v{{ availableUpdate.currentVersion }} · 发布于 {{ formattedDate }}</p>
        </div>
      </div>

      <section class="app-update-dialog__notes">
        <h4>更新内容</h4>
        <p>{{ availableUpdate.notes?.trim() || '本次版本包含功能改进与稳定性优化。' }}</p>
      </section>

      <div
        v-if="statusText"
        class="app-update-dialog__status"
        :class="{ 'app-update-dialog__status--error': phase === 'failed' }"
      >
        <TriangleAlert v-if="phase === 'failed'" :size="18" />
        <RefreshCw v-else class="app-update-dialog__spinner" :size="18" />
        <div>
          <strong>{{ statusText }}</strong>
          <p v-if="lastError">{{ lastError }}</p>
        </div>
      </div>

      <div v-if="phase === 'downloading'" class="app-update-dialog__progress" aria-label="更新下载进度">
        <span
          :class="{ 'app-update-dialog__progress-bar--indeterminate': downloadProgress === null }"
          :style="downloadProgress === null ? undefined : { width: `${downloadProgress}%` }"
        />
      </div>
    </div>

    <template #footer>
      <BaseButton v-if="phase === 'failed'" variant="ghost" @click="updater.openReleasesPage">前往发布页</BaseButton>
      <BaseButton variant="secondary" :disabled="isInstalling" @click="updater.dismissUpdate">以后更新</BaseButton>
      <BaseButton variant="primary" :loading="isInstalling" @click="updater.installUpdate">
        <template #icon><Download :size="16" /></template>
        {{ phase === 'failed' ? '重新尝试' : '立即更新' }}
      </BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .app-update-dialog {
    display: grid;
    gap: 18px;
  }

  .app-update-dialog__hero {
    display: flex;
    gap: 14px;
    align-items: center;
  }

  .app-update-dialog__icon {
    display: grid;
    width: 50px;
    height: 50px;
    flex: 0 0 auto;
    border: 1px solid var(--accent-border);
    border-radius: 12px;
    background: var(--accent-soft);
    color: var(--accent-strong);
    place-items: center;
  }

  .app-update-dialog h3,
  .app-update-dialog h4,
  .app-update-dialog p {
    margin: 0;
  }

  .app-update-dialog h3 {
    margin: 2px 0 4px;
    font-size: 24px;
  }

  .app-update-dialog__eyebrow {
    color: var(--accent-strong);
    font-size: var(--font-size-xs);
    font-weight: 700;
  }

  .app-update-dialog__hero p:last-child,
  .app-update-dialog__status p {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .app-update-dialog__notes {
    display: grid;
    gap: 9px;
    max-height: 220px;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    padding: 14px;
  }

  .app-update-dialog__notes h4,
  .app-update-dialog__notes p {
    font-size: var(--font-size-sm);
  }

  .app-update-dialog__notes p {
    color: var(--text-muted);
    line-height: 1.7;
    white-space: pre-wrap;
  }

  .app-update-dialog__status {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--accent-strong);
    padding: 11px 12px;
  }

  .app-update-dialog__status--error {
    border-color: rgba(248, 113, 113, 0.26);
    background: var(--danger-soft);
    color: #fca5a5;
  }

  .app-update-dialog__status strong {
    color: var(--text);
    font-size: var(--font-size-sm);
  }

  .app-update-dialog__spinner {
    animation: update-spin 900ms linear infinite;
  }

  .app-update-dialog__progress {
    height: 6px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
  }

  .app-update-dialog__progress span {
    display: block;
    width: 0;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, #8d73ff, #c4b5fd);
    transition: width 160ms ease;
  }

  .app-update-dialog__progress-bar--indeterminate {
    width: 42% !important;
    animation: update-progress 1.1s ease-in-out infinite;
  }

  @keyframes update-spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes update-progress {
    from {
      transform: translateX(-110%);
    }

    to {
      transform: translateX(260%);
    }
  }
</style>
