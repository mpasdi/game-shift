<script setup lang="ts">
  import { computed, reactive, ref, watch } from 'vue'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { open as openDialog } from '@tauri-apps/plugin-dialog'
  import { FileSearch, FolderOpen } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import TextField from '../../../shared/components/TextField.vue'
  import type { CreateGamePayload, Game, UpdateGamePayload } from '../types/game'

  const props = withDefaults(
    defineProps<{
      open: boolean
      saving: boolean
      mode?: 'create' | 'edit'
      game?: Game | null
      errorMessage?: string | null
    }>(),
    {
      mode: 'create',
      game: null,
      errorMessage: null
    }
  )

  const emit = defineEmits<{
    close: []
    submit: [payload: CreateGamePayload | UpdateGamePayload]
  }>()

  const form = reactive({
    name: '',
    exePath: '',
    workDir: '',
    args: ''
  })
  const localError = ref<string | null>(null)

  const isEditing = computed(() => props.mode === 'edit')
  const modalTitle = computed(() => (isEditing.value ? '编辑游戏' : '手动添加游戏'))
  const submitText = computed(() => (isEditing.value ? '保存修改' : '保存'))
  const displayedError = computed(() => localError.value || props.errorMessage || null)
  const previewInitial = computed(() => form.name.trim().slice(0, 1).toUpperCase() || 'G')
  const previewCoverSrc = computed(() => toLocalAssetSrc(props.game?.cover))
  const previewIconSrc = computed(() => toLocalAssetSrc(props.game?.icon))
  const previewHint = computed(() => (isEditing.value ? '自动识别封面' : '保存后自动识别'))

  watch(
    () => [props.open, props.game, props.mode] as const,
    ([open]) => {
      if (!open) {
        resetForm()
        return
      }

      if (isEditing.value && props.game) {
        fillForm(props.game)
        return
      }

      resetForm()
    },
    { immediate: true }
  )

  async function chooseExePath() {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: 'Windows 可执行文件', extensions: ['exe'] }]
    })

    if (typeof selected !== 'string') return

    form.exePath = selected
    form.workDir = getFolderPath(selected)
    if (!form.name.trim()) form.name = inferGameName(selected)
  }

  async function chooseWorkDir() {
    const selected = await openDialog({
      multiple: false,
      directory: true,
      defaultPath: form.workDir || getFolderPath(form.exePath) || undefined
    })

    if (typeof selected !== 'string') return

    form.workDir = selected
  }

  function toLocalAssetSrc(path?: string | null) {
    return path ? convertFileSrc(path) : null
  }

  function submitForm() {
    localError.value = validateForm()
    if (localError.value) return

    const basePayload = {
      name: form.name.trim(),
      exePath: form.exePath.trim(),
      workDir: form.workDir.trim() || null,
      args: form.args.trim() || null
    }

    if (isEditing.value) {
      if (!props.game) {
        localError.value = '缺少要编辑的游戏记录'
        return
      }
      emit('submit', {
        ...basePayload,
        id: props.game.id,
        favorite: props.game.favorite
      })
      return
    }

    emit('submit', basePayload)
  }

  function fillForm(game: Game) {
    form.name = game.name
    form.exePath = game.exePath
    form.workDir = game.workDir ?? game.folderPath
    form.args = game.args ?? ''
    localError.value = null
  }

  function resetForm() {
    form.name = ''
    form.exePath = ''
    form.workDir = ''
    form.args = ''
    localError.value = null
  }

  function validateForm() {
    if (!form.exePath.trim()) return '请选择游戏启动程序'
    if (!form.exePath.trim().toLowerCase().endsWith('.exe')) return '启动程序必须是 .exe 文件'
    if (!form.name.trim()) return '游戏名称不能为空'
    return null
  }

  function inferGameName(path: string) {
    const folderPath = getFolderPath(path)
    const folderName = getFileName(folderPath)
    if (folderName) return folderName
    return getFileName(path).replace(/\.exe$/i, '')
  }

  function getFolderPath(path: string) {
    const index = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
    return index >= 0 ? path.slice(0, index) : ''
  }

  function getFileName(path: string) {
    const index = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
    return index >= 0 ? path.slice(index + 1) : path
  }
</script>

<template>
  <BaseModal :open="props.open" :title="modalTitle" size="lg" @close="emit('close')">
    <div class="game-dialog">
      <form class="game-form" @submit.prevent="submitForm">
        <div class="game-form__hero">
          <aside class="cover-preview" aria-label="游戏封面预览">
            <div class="cover-preview__art" aria-hidden="true">
              <img v-if="previewCoverSrc" class="cover-preview__image" :src="previewCoverSrc" alt="" />
              <img v-else-if="previewIconSrc" class="cover-preview__icon" :src="previewIconSrc" alt="" />
              <span v-else>{{ previewInitial }}</span>
              <span class="cover-preview__hint">{{ previewHint }}</span>
            </div>
          </aside>

          <div class="game-form__hero-fields">
            <div class="form-field">
              <label class="form-field__label" for="game-name">游戏名称</label>
              <TextField id="game-name" v-model="form.name" placeholder="例如：Elden Ring" />
            </div>

            <div class="form-field">
              <label class="form-field__label" for="game-exe">可执行文件</label>
              <div class="path-field">
                <TextField id="game-exe" v-model="form.exePath" placeholder="选择游戏 .exe 文件" readonly />
                <BaseButton variant="secondary" type="button" @click="chooseExePath">
                  <template #icon><FileSearch :size="17" /></template>
                  选择
                </BaseButton>
              </div>
            </div>

            <div class="form-field">
              <label class="form-field__label" for="game-work-dir">工作目录</label>
              <div class="path-field">
                <TextField id="game-work-dir" v-model="form.workDir" placeholder="默认使用 .exe 所在目录" readonly />
                <BaseButton variant="secondary" type="button" @click="chooseWorkDir">
                  <template #icon><FolderOpen :size="17" /></template>
                  选择
                </BaseButton>
              </div>
            </div>
          </div>
        </div>

        <div class="form-field form-field--full">
          <label class="form-field__label" for="game-args">启动参数</label>
          <TextField id="game-args" v-model="form.args" placeholder="可选，例如 -windowed" />
        </div>

        <p v-if="displayedError" class="form-error">{{ displayedError }}</p>
      </form>
    </div>

    <template #footer>
      <BaseButton variant="primary" type="button" :loading="saving" @click="submitForm">{{ submitText }}</BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .game-form {
    display: grid;
    gap: 16px;
  }

  .game-dialog {
    min-width: 0;
  }

  .game-form__hero {
    display: grid;
    grid-template-columns: 132px minmax(0, 1fr);
    gap: 20px;
    align-items: stretch;
  }

  .game-form__hero-fields {
    display: grid;
    align-content: center;
    gap: 14px;
    min-width: 0;
  }

  .cover-preview {
    display: grid;
  }

  .cover-preview__art {
    display: grid;
    position: relative;
    overflow: hidden;
    width: 132px;
    height: 100%;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background:
      radial-gradient(circle at 50% 20%, rgba(157, 140, 255, 0.26), transparent 42%),
      linear-gradient(145deg, rgba(124, 92, 255, 0.32), rgba(255, 255, 255, 0.055));
    color: var(--text);
    font-size: 42px;
    font-weight: 800;
  }

  .cover-preview__image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-preview__icon {
    width: 64%;
    height: 64%;
    object-fit: contain;
  }

  .cover-preview__hint {
    position: absolute;
    right: 8px;
    bottom: 8px;
    left: 8px;
    overflow: hidden;
    border-radius: 5px;
    background: rgba(12, 10, 18, 0.34);
    color: var(--text-subtle);
    padding: 3px 6px;
    font-size: var(--font-size-xs);
    font-weight: 500;
    line-height: 1.2;
    text-align: center;
    text-overflow: ellipsis;
    white-space: nowrap;
    backdrop-filter: blur(8px);
  }

  .form-field {
    display: grid;
    gap: 7px;
    min-width: 0;
  }

  .form-field--full :deep(.text-field) {
    width: 100%;
  }

  .form-field__label {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 600;
    line-height: 1.2;
  }

  .path-field {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
  }

  .form-error {
    margin: 0;
    color: #fecdd3;
    font-size: var(--font-size-md);
  }

  @media (max-width: 720px) {
    .game-form__hero {
      grid-template-columns: 112px minmax(0, 1fr);
      gap: 14px;
      align-items: stretch;
    }

    .cover-preview__art {
      width: 112px;
      height: 150px;
    }

    .path-field {
      align-items: stretch;
      grid-template-columns: 1fr;
    }

    .path-field :deep(.base-button) {
      width: 100%;
    }
  }
</style>
