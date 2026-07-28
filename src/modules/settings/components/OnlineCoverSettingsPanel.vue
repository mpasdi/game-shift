<script setup lang="ts">
  import { computed, onMounted, ref } from 'vue'
  import { Image as ImageIcon, KeyRound, ShieldCheck, Trash2, TriangleAlert, Wifi } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import TextField from '../../../shared/components/TextField.vue'
  import { getErrorMessage, useToast } from '../../../shared/composables/useToast'
  import {
    deleteSteamGridDbApiKey,
    getOnlineCoverSettings,
    saveSteamGridDbApiKey,
    setOnlineCoversEnabled,
    testSteamGridDbConnection,
    type OnlineCoverConfigState,
    type OnlineCoverSettings
  } from '../api'

  type Operation = 'toggle' | 'save' | 'test' | 'delete'

  const toast = useToast()
  const settings = ref<OnlineCoverSettings | null>(null)
  const apiKeyInput = ref('')
  const isEditingApiKey = ref(false)
  const isDeleteConfirmOpen = ref(false)
  const isLoading = ref(false)
  const activeOperation = ref<Operation | null>(null)
  const loadError = ref<string | null>(null)

  const statusContent: Record<OnlineCoverConfigState, { label: string; description: string }> = {
    disabled: {
      label: '未启用',
      description: '联网入口保持隐藏，现有本地封面不受影响。'
    },
    missingApiKey: {
      label: '已启用但未配置',
      description: '保存 SteamGridDB API Key 后才能使用联网封面。'
    },
    ready: {
      label: '已启用且已配置',
      description: 'API Key 已保存在本机，可以测试连接或继续后续联网搜索。'
    },
    invalidApiKey: {
      label: '配置异常',
      description: 'SteamGridDB 明确拒绝了当前 Key，请替换后重新测试。'
    }
  }

  const currentStatus = computed(() => statusContent[settings.value?.state ?? 'disabled'])
  const isBusy = computed(() => activeOperation.value !== null)
  const canSaveKey = computed(() => apiKeyInput.value.trim().length > 0 && !isBusy.value)
  const storedKeyStatusLabel = computed(() => (settings.value?.state === 'invalidApiKey' ? '验证失败' : '验证通过'))

  async function loadSettings() {
    isLoading.value = true
    loadError.value = null
    try {
      settings.value = await getOnlineCoverSettings()
    } catch (error) {
      loadError.value = getErrorMessage(error)
    } finally {
      isLoading.value = false
    }
  }

  async function toggleOnlineCovers() {
    if (!settings.value || isBusy.value) return
    activeOperation.value = 'toggle'
    try {
      settings.value = await setOnlineCoversEnabled(!settings.value.enabled)
      toast.success({ title: settings.value.enabled ? '联网封面已启用' : '联网封面已关闭' })
    } catch (error) {
      toast.error({ title: '更新联网封面设置失败', description: getErrorMessage(error) })
    } finally {
      activeOperation.value = null
    }
  }

  async function saveApiKey() {
    const apiKey = apiKeyInput.value.trim()
    if (!apiKey || isBusy.value) return
    activeOperation.value = 'save'
    try {
      settings.value = await saveSteamGridDbApiKey(apiKey)
      apiKeyInput.value = ''
      isEditingApiKey.value = false
      toast.success({ title: 'SteamGridDB API Key 验证成功并已保存' })
    } catch (error) {
      toast.error({ title: '保存失败', description: getErrorMessage(error) })
    } finally {
      activeOperation.value = null
    }
  }

  async function testConnection() {
    if (!settings.value?.hasApiKey || isBusy.value) return
    activeOperation.value = 'test'
    try {
      settings.value = await testSteamGridDbConnection()
      toast.success({ title: 'SteamGridDB 连接成功', description: '当前 API Key 可以正常使用' })
    } catch (error) {
      await refreshAfterFailedTest()
      toast.error({ title: 'SteamGridDB 连接失败', description: getErrorMessage(error) })
    } finally {
      activeOperation.value = null
    }
  }

  async function refreshAfterFailedTest() {
    try {
      settings.value = await getOnlineCoverSettings()
    } catch {
      // 保留当前界面状态，主要错误由测试连接操作反馈。
    }
  }

  async function deleteApiKey() {
    if (!settings.value?.hasApiKey || isBusy.value) return

    activeOperation.value = 'delete'
    try {
      settings.value = await deleteSteamGridDbApiKey()
      apiKeyInput.value = ''
      isEditingApiKey.value = false
      isDeleteConfirmOpen.value = false
      toast.success({ title: 'SteamGridDB API Key 已删除' })
    } catch (error) {
      toast.error({ title: '删除 API Key 失败', description: getErrorMessage(error) })
    } finally {
      activeOperation.value = null
    }
  }

  function beginApiKeyEdit() {
    apiKeyInput.value = ''
    isEditingApiKey.value = true
  }

  function cancelApiKeyEdit() {
    apiKeyInput.value = ''
    isEditingApiKey.value = false
  }

  function openDeleteConfirm() {
    if (!settings.value?.hasApiKey || isBusy.value) return
    isDeleteConfirmOpen.value = true
  }

  function closeDeleteConfirm() {
    if (activeOperation.value === 'delete') return
    isDeleteConfirmOpen.value = false
  }

  onMounted(() => {
    void loadSettings()
  })
</script>

<template>
  <article class="online-cover-panel">
    <div class="online-cover-panel__heading">
      <div class="online-cover-panel__title">
        <ImageIcon :size="16" />
        <div>
          <h2>联网封面</h2>
          <p>可选的 SteamGridDB 封面数据源</p>
        </div>
      </div>

      <button
        class="online-cover-switch"
        type="button"
        role="switch"
        :aria-checked="settings?.enabled ?? false"
        :aria-label="settings?.enabled ? '关闭联网封面' : '启用联网封面'"
        :disabled="!settings || isLoading || isBusy"
        @click="toggleOnlineCovers"
      >
        <span class="online-cover-switch__track" aria-hidden="true">
          <span class="online-cover-switch__thumb" />
        </span>
        <span>{{ settings?.enabled ? '已开启' : '已关闭' }}</span>
      </button>
    </div>

    <div v-if="isLoading && !settings" class="online-cover-panel__placeholder">正在读取联网封面设置...</div>

    <div v-else-if="loadError && !settings" class="online-cover-panel__load-error">
      <span>{{ loadError }}</span>
      <BaseButton size="sm" @click="loadSettings">重新读取</BaseButton>
    </div>

    <div v-else-if="settings" class="online-cover-panel__body">
      <div class="online-cover-status" :class="`online-cover-status--${settings.state}`">
        <ShieldCheck v-if="settings.state === 'ready'" :size="17" />
        <TriangleAlert
          v-else-if="settings.state === 'missingApiKey' || settings.state === 'invalidApiKey'"
          :size="17"
        />
        <Wifi v-else :size="17" />
        <div>
          <strong>{{ currentStatus.label }}</strong>
          <p>{{ currentStatus.description }}</p>
        </div>
      </div>

      <div v-if="settings.enabled" class="online-cover-config">
        <div class="online-cover-notice">
          <p>联网搜索由 SteamGridDB 提供，仅发送搜索词和 API Key。</p>
        </div>

        <section class="api-key-config">
          <h3>SteamGridDB API Key</h3>

          <div v-if="settings.hasApiKey && !isEditingApiKey" class="api-key-summary">
            <div class="api-key-summary__identity">
              <KeyRound :size="16" />
              <strong>{{ settings.apiKeyHint }}</strong>
              <span
                class="api-key-summary__status"
                :class="{ 'api-key-summary__status--invalid': settings.state === 'invalidApiKey' }"
              >
                {{ storedKeyStatusLabel }}
              </span>
            </div>

            <div class="api-key-summary__actions">
              <BaseButton
                variant="ghost"
                size="sm"
                :loading="activeOperation === 'test'"
                :disabled="isBusy"
                @click="testConnection"
              >
                重新验证
              </BaseButton>
              <BaseButton variant="ghost" size="sm" :disabled="isBusy" @click="beginApiKeyEdit">更换</BaseButton>
              <BaseButton
                class="api-key-delete-trigger"
                variant="ghost"
                size="sm"
                :disabled="isBusy"
                @click="openDeleteConfirm"
              >
                删除
              </BaseButton>
            </div>
          </div>

          <form v-else class="api-key-form" @submit.prevent="saveApiKey">
            <div class="api-key-form__controls">
              <TextField
                id="steamgriddb-api-key"
                v-model="apiKeyInput"
                type="password"
                autocomplete="off"
                :placeholder="settings.hasApiKey ? '粘贴新的 API Key' : '粘贴 SteamGridDB API Key'"
              >
                <template #icon><KeyRound :size="15" /></template>
              </TextField>
              <BaseButton variant="primary" type="submit" :loading="activeOperation === 'save'" :disabled="!canSaveKey">
                验证并保存
              </BaseButton>
              <BaseButton v-if="settings.hasApiKey" variant="ghost" :disabled="isBusy" @click="cancelApiKeyEdit">
                取消
              </BaseButton>
            </div>
          </form>
        </section>
      </div>
    </div>
  </article>

  <BaseModal :open="isDeleteConfirmOpen" title="删除 API Key" size="sm" @close="closeDeleteConfirm">
    <div class="api-key-delete-confirm">
      <div class="api-key-delete-confirm__icon" aria-hidden="true">
        <Trash2 :size="30" />
      </div>
      <div>
        <p class="api-key-delete-confirm__title">删除 SteamGridDB API Key？</p>
        <p class="api-key-delete-confirm__description">删除后将无法使用联网封面，联网封面开关保持开启。</p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" :disabled="activeOperation === 'delete'" @click="closeDeleteConfirm">
        取消
      </BaseButton>
      <BaseButton variant="danger" :loading="activeOperation === 'delete'" @click="deleteApiKey">确认删除</BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .online-cover-panel {
    min-width: 0;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.24);
  }

  .online-cover-panel__heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
    padding: 14px 16px;
  }

  .online-cover-panel__title {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }

  .online-cover-panel__title > svg {
    margin-top: 1px;
    color: var(--accent-strong);
  }

  .online-cover-panel h2,
  .online-cover-panel p {
    margin: 0;
  }

  .online-cover-panel h2 {
    font-size: var(--font-size-md);
    line-height: 1.2;
  }

  .online-cover-panel__title p {
    margin-top: 4px;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
  }

  .online-cover-switch {
    display: inline-flex;
    gap: 8px;
    align-items: center;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 4px;
    font-size: var(--font-size-sm);
  }

  .online-cover-switch__track {
    display: flex;
    width: 34px;
    height: 19px;
    align-items: center;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    background: var(--surface);
    padding: 2px;
    transition: background 160ms ease;
  }

  .online-cover-switch__thumb {
    width: 13px;
    height: 13px;
    border-radius: 999px;
    background: var(--text-muted);
    transition:
      background 160ms ease,
      transform 160ms ease;
  }

  .online-cover-switch[aria-checked='true'] {
    color: var(--text);
  }

  .online-cover-switch[aria-checked='true'] .online-cover-switch__track {
    border-color: var(--accent-border);
    background: var(--accent);
  }

  .online-cover-switch[aria-checked='true'] .online-cover-switch__thumb {
    background: #ffffff;
    transform: translateX(15px);
  }

  .online-cover-switch:focus-visible {
    border-radius: 7px;
    outline: 3px solid var(--focus-ring);
  }

  .online-cover-switch:disabled {
    opacity: 0.5;
  }

  .online-cover-panel__placeholder,
  .online-cover-panel__load-error {
    display: flex;
    min-height: 92px;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    padding: 18px;
    font-size: var(--font-size-sm);
  }

  .online-cover-panel__load-error {
    gap: 12px;
    color: #fecaca;
  }

  .online-cover-panel__body {
    display: grid;
    gap: 14px;
    padding: 14px 16px 16px;
  }

  .online-cover-status {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.026);
    padding: 10px 12px;
    color: var(--text-muted);
  }

  .online-cover-status strong {
    display: block;
    color: var(--text);
    font-size: var(--font-size-sm);
  }

  .online-cover-status p {
    margin-top: 3px;
    font-size: var(--font-size-xs);
    line-height: 1.5;
  }

  .online-cover-status--ready {
    border-color: rgba(74, 222, 128, 0.24);
    background: rgba(34, 197, 94, 0.08);
  }

  .online-cover-status--ready > svg {
    color: #4ade80;
  }

  .online-cover-status--missingApiKey,
  .online-cover-status--invalidApiKey {
    border-color: rgba(248, 113, 113, 0.22);
    background: var(--danger-soft);
  }

  .online-cover-status--missingApiKey > svg,
  .online-cover-status--invalidApiKey > svg {
    color: var(--danger);
  }

  .online-cover-config {
    display: grid;
    gap: 14px;
  }

  .online-cover-notice {
    display: grid;
    gap: 5px;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.55;
  }

  .api-key-config {
    display: grid;
    gap: 7px;
  }

  .api-key-config h3 {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  .api-key-summary {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    min-height: 34px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    padding: 1px 7px 1px 12px;
  }

  .api-key-summary__identity,
  .api-key-summary__actions {
    display: flex;
    align-items: center;
  }

  .api-key-summary__identity {
    flex: 1;
    gap: 8px;
    min-width: 0;
  }

  .api-key-summary__identity > svg {
    flex: 0 0 auto;
    color: var(--text-subtle);
  }

  .api-key-summary__identity strong {
    overflow: hidden;
    color: var(--text);
    font-size: var(--font-size-sm);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .api-key-summary__status {
    flex: 0 0 auto;
    border-radius: 999px;
    background: rgba(34, 197, 94, 0.12);
    color: #4ade80;
    padding: 2px 7px;
    font-size: var(--font-size-xs);
  }

  .api-key-summary__status--invalid {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .api-key-summary__actions {
    gap: 2px;
    flex: 0 0 auto;
  }

  .api-key-summary__actions :deep(.api-key-delete-trigger) {
    color: var(--danger);
  }

  .api-key-summary__actions :deep(.api-key-delete-trigger:hover:not(:disabled)) {
    border-color: rgba(248, 113, 113, 0.28);
    background: var(--danger-soft);
    color: #fda4af;
  }

  .api-key-form {
    display: grid;
  }

  .api-key-form__controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .api-key-form__controls :deep(.text-field) {
    flex: 1;
  }

  .api-key-delete-confirm {
    display: grid;
    gap: 16px;
    justify-items: center;
    padding: 8px 6px 4px;
    text-align: center;
  }

  .api-key-delete-confirm__icon {
    display: grid;
    width: 68px;
    height: 68px;
    place-items: center;
    border: 1px solid rgba(248, 113, 113, 0.45);
    border-radius: 999px;
    background: var(--danger-soft);
    color: var(--danger);
  }

  .api-key-delete-confirm__title,
  .api-key-delete-confirm__description {
    margin: 0;
  }

  .api-key-delete-confirm__title {
    color: var(--text);
    font-weight: 700;
  }

  .api-key-delete-confirm__description {
    margin-top: 8px;
    color: var(--text-subtle);
    font-size: var(--font-size-sm);
    line-height: 1.5;
  }

  @media (max-width: 620px) {
    .online-cover-panel__heading,
    .api-key-summary,
    .api-key-form__controls {
      align-items: stretch;
    }

    .api-key-summary,
    .api-key-form__controls {
      flex-direction: column;
    }

    .api-key-summary__identity {
      min-height: 30px;
    }

    .api-key-summary__actions {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
    }

    .api-key-form__controls :deep(.base-button),
    .api-key-summary__actions :deep(.base-button) {
      width: 100%;
    }
  }
</style>
