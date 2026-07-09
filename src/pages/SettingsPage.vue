<script setup lang="ts">
  import { computed, onMounted, ref } from 'vue'
  import { AppWindow, Database, FolderOpen, Info, Settings } from '@lucide/vue'
  import { getAppInfo, type AppInfo } from '../modules/settings/api'

  const appInfo = ref<AppInfo | null>(null)
  const isLoading = ref(false)
  const errorMessage = ref<string | null>(null)

  const infoItems = computed(() => {
    if (!appInfo.value) return []

    return [
      {
        label: '应用名称',
        value: appInfo.value.name,
        icon: AppWindow
      },
      {
        label: '当前版本',
        value: appInfo.value.version,
        icon: Info
      },
      {
        label: '应用标识',
        value: appInfo.value.identifier,
        icon: Settings
      },
      {
        label: '数据目录',
        value: appInfo.value.dataDir,
        icon: FolderOpen
      },
      {
        label: '数据库文件',
        value: appInfo.value.databasePath,
        icon: Database
      }
    ]
  })

  async function loadAppInfo() {
    isLoading.value = true
    errorMessage.value = null

    try {
      appInfo.value = await getAppInfo()
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error)
    } finally {
      isLoading.value = false
    }
  }

  onMounted(() => {
    void loadAppInfo()
  })
</script>

<template>
  <section class="settings-page">
    <header class="settings-page__header">
      <div>
        <h1>设置</h1>
      </div>
    </header>

    <p v-if="errorMessage" class="settings-page__error">{{ errorMessage }}</p>

    <div class="settings-grid">
      <article class="settings-panel">
        <div class="settings-panel__heading">
          <Settings :size="16" />
          <h2>应用信息</h2>
        </div>

        <div class="info-list" :class="{ 'info-list--loading': isLoading && !appInfo }">
          <div v-for="item in infoItems" :key="item.label" class="info-row">
            <span class="info-row__icon"><component :is="item.icon" :size="15" /></span>
            <span class="info-row__label">{{ item.label }}</span>
            <span class="info-row__value" :title="item.value">{{ item.value }}</span>
          </div>
          <div v-if="!appInfo && isLoading" class="info-placeholder">正在读取应用信息...</div>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
  .settings-page {
    --settings-content-width: 720px;
    width: min(100%, 1120px);
    margin: 0 auto;
    padding: 22px 0 40px;
  }

  .settings-page__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: min(100%, var(--settings-content-width));
    margin: 0 auto 18px;
  }

  .settings-page h1 {
    margin: 0;
    font-size: var(--font-size-xl);
    line-height: 1.2;
  }

  .settings-page__error {
    width: min(100%, var(--settings-content-width));
    margin: 0 auto 14px;
    border: 1px solid rgba(248, 113, 113, 0.25);
    border-radius: 8px;
    background: var(--danger-soft);
    color: #fecaca;
    padding: 10px 12px;
    font-size: var(--font-size-sm);
  }

  .settings-grid {
    display: grid;
    width: min(100%, var(--settings-content-width));
    margin: 0 auto;
  }

  .settings-panel {
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.24);
  }

  .settings-panel__heading {
    display: flex;
    gap: 8px;
    align-items: center;
    border-bottom: 1px solid var(--border);
    padding: 14px 16px;
    color: var(--text);
  }

  .settings-panel__heading svg {
    color: var(--accent-strong);
  }

  .settings-panel h2 {
    margin: 0;
    font-size: var(--font-size-md);
    line-height: 1.2;
  }

  .info-list {
    padding: 6px 0;
  }

  .info-list--loading {
    min-height: 180px;
  }

  .info-row {
    display: grid;
    grid-template-columns: 30px 88px minmax(0, 1fr);
    gap: 10px;
    align-items: center;
    min-height: 48px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.055);
    padding: 0 16px;
  }

  .info-row:last-child {
    border-bottom: 0;
  }

  .info-row__icon {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 7px;
    background: var(--accent-soft);
    color: var(--accent-strong);
  }

  .info-row__label {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .info-row__value {
    overflow: hidden;
    color: var(--text);
    font-size: var(--font-size-sm);
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .info-placeholder {
    display: grid;
    min-height: 180px;
    place-items: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  @media (max-width: 860px) {
    .settings-page__header {
      align-items: flex-start;
      gap: 12px;
    }
  }
</style>
