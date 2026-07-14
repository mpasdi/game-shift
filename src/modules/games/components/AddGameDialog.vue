<script setup lang="ts">
  import { computed, reactive, ref, watch } from 'vue'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { open as openDialog } from '@tauri-apps/plugin-dialog'
  import { FileSearch, FolderOpen, ImagePlus } from '@lucide/vue'
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
    args: '',
    coverPath: ''
  })
  const localError = ref<string | null>(null)

  const isEditing = computed(() => props.mode === 'edit')
  const modalTitle = computed(() => (isEditing.value ? '编辑游戏' : '手动添加游戏'))
  const submitText = computed(() => (isEditing.value ? '保存修改' : '保存'))
  const displayedError = computed(() => localError.value || props.errorMessage || null)
  const previewInitial = computed(() => form.name.trim().slice(0, 1).toUpperCase() || 'G')
  const previewCoverSrc = computed(() => toLocalAssetSrc(form.coverPath || props.game?.cover))
  const previewIconSrc = computed(() => toLocalAssetSrc(props.game?.icon))
  const coverActionText = computed(() => (previewCoverSrc.value ? '更换封面' : '选择封面'))

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

  async function chooseCoverPath() {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: '游戏封面', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]
    })

    if (typeof selected !== 'string') return

    form.coverPath = selected
    localError.value = null
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
      args: form.args.trim() || null,
      coverPath: form.coverPath.trim() || null
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
    form.coverPath = ''
    localError.value = null
  }

  function resetForm() {
    form.name = ''
    form.exePath = ''
    form.workDir = ''
    form.args = ''
    form.coverPath = ''
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
            <button
              class="cover-preview__art"
              :class="{ 'cover-preview__art--empty': !previewCoverSrc && !previewIconSrc }"
              type="button"
              :aria-label="coverActionText"
              :disabled="saving"
              @click="chooseCoverPath"
            >
              <img
                v-if="previewCoverSrc"
                :key="previewCoverSrc"
                class="cover-preview__image"
                :src="previewCoverSrc"
                alt=""
              />
              <img v-else-if="previewIconSrc" class="cover-preview__icon" :src="previewIconSrc" alt="" />
              <span v-else>{{ previewInitial }}</span>
              <span class="cover-preview__overlay" aria-hidden="true">
                <span class="cover-preview__action">
                  <ImagePlus :size="17" />
                  {{ coverActionText }}
                </span>
              </span>
            </button>
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
      <BaseButton variant="secondary" type="button" :disabled="saving" @click="emit('close')">取消</BaseButton>
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
    display: block;
  }

  .cover-preview__art {
    display: grid;
    position: relative;
    overflow: hidden;
    width: 132px;
    aspect-ratio: 2 / 3;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    padding: 0;
    background:
      radial-gradient(circle at 50% 20%, rgba(157, 140, 255, 0.26), transparent 42%),
      linear-gradient(145deg, rgba(124, 92, 255, 0.32), rgba(255, 255, 255, 0.055));
    color: var(--text);
    cursor: pointer;
    font-size: 42px;
    font-family: inherit;
    font-weight: 800;
  }

  .cover-preview__art:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 3px;
  }

  .cover-preview__art:disabled {
    cursor: not-allowed;
    opacity: 0.72;
  }

  .cover-preview__image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 180ms ease;
  }

  .cover-preview__icon {
    width: 64%;
    height: 64%;
    object-fit: contain;
  }

  .cover-preview__overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(10, 8, 15, 0.58);
    opacity: 0;
    transition: opacity 180ms ease;
  }

  .cover-preview__action {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    justify-content: center;
    padding: 8px;
    color: #fff;
    font-size: var(--font-size-sm);
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.72);
  }

  .cover-preview__art:hover:not(:disabled) .cover-preview__overlay,
  .cover-preview__art:focus-visible .cover-preview__overlay,
  .cover-preview__art--empty .cover-preview__overlay {
    opacity: 1;
  }

  .cover-preview__art:hover:not(:disabled) .cover-preview__image {
    transform: scale(1.025);
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
