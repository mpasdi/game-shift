<script setup lang="ts">
  import { computed, reactive, ref, watch } from 'vue'
  import { open as openDialog } from '@tauri-apps/plugin-dialog'
  import { FileSearch, FolderOpen, Globe2, ImagePlus } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import TextField from '../../../shared/components/TextField.vue'
  import { getOnlineCoverSettings } from '../../settings/api'
  import GameArtwork from './GameArtwork.vue'
  import OnlineCoverDialog from './OnlineCoverDialog.vue'
  import type { CoverCandidate, CoverSelection } from '../types/cover'
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
  const onlineCoverAvailable = ref(false)
  const isOnlineCoverDialogOpen = ref(false)
  const selectedRemoteCover = ref<CoverCandidate | null>(null)

  const isEditing = computed(() => props.mode === 'edit')
  const modalTitle = computed(() => (isEditing.value ? '编辑游戏' : '手动添加游戏'))
  const submitText = computed(() => (isEditing.value ? '保存修改' : '保存'))
  const displayedError = computed(() => localError.value || props.errorMessage || null)
  const previewGame = computed(() => ({
    name: form.name || props.game?.name || '',
    cover: selectedRemoteCover.value?.previewUrl || form.coverPath || props.game?.cover || null,
    icon: props.game?.icon || null
  }))

  watch(
    () => [props.open, props.game, props.mode] as const,
    ([open]) => {
      if (!open) {
        resetForm()
        return
      }

      if (isEditing.value && props.game) {
        fillForm(props.game)
        void loadOnlineCoverAvailability()
        return
      }

      resetForm()
      void loadOnlineCoverAvailability()
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
    selectedRemoteCover.value = null
    localError.value = null
  }

  function openOnlineCoverDialog() {
    isOnlineCoverDialogOpen.value = true
  }

  function selectRemoteCover(candidate: CoverCandidate) {
    selectedRemoteCover.value = candidate
    form.coverPath = ''
    isOnlineCoverDialogOpen.value = false
    localError.value = null
  }

  function submitForm() {
    localError.value = validateForm()
    if (localError.value) return

    const basePayload = {
      name: form.name.trim(),
      exePath: form.exePath.trim(),
      workDir: form.workDir.trim() || null,
      args: form.args.trim() || null,
      coverPath: null,
      coverSelection: getCoverSelection()
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
    selectedRemoteCover.value = null
    isOnlineCoverDialogOpen.value = false
    localError.value = null
  }

  function resetForm() {
    form.name = ''
    form.exePath = ''
    form.workDir = ''
    form.args = ''
    form.coverPath = ''
    selectedRemoteCover.value = null
    isOnlineCoverDialogOpen.value = false
    localError.value = null
  }

  function getCoverSelection(): CoverSelection {
    if (selectedRemoteCover.value) {
      return {
        type: 'remote',
        provider: selectedRemoteCover.value.provider,
        providerGameId: selectedRemoteCover.value.providerGameId,
        assetId: selectedRemoteCover.value.assetId
      }
    }
    if (form.coverPath.trim()) {
      return { type: 'local', path: form.coverPath.trim() }
    }
    return { type: 'unchanged' }
  }

  async function loadOnlineCoverAvailability() {
    onlineCoverAvailable.value = false
    try {
      const settings = await getOnlineCoverSettings()
      onlineCoverAvailable.value = settings.state === 'ready'
    } catch {
      onlineCoverAvailable.value = false
    }
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
            <span class="form-field__label">游戏封面</span>
            <div
              class="cover-preview__art"
              :class="{ 'cover-preview__art--empty': !previewGame.cover && !previewGame.icon }"
            >
              <GameArtwork :game="previewGame" variant="preview" />
            </div>
            <div class="cover-preview__actions">
              <BaseButton
                class="cover-preview__action"
                variant="secondary"
                size="sm"
                type="button"
                title="选择本地封面"
                :disabled="saving"
                @click="chooseCoverPath"
              >
                <template #icon><ImagePlus :size="15" /></template>
                本地
              </BaseButton>
              <BaseButton
                v-if="onlineCoverAvailable"
                class="cover-preview__action"
                variant="secondary"
                size="sm"
                type="button"
                title="联网搜索封面"
                :disabled="saving"
                @click="openOnlineCoverDialog"
              >
                <template #icon><Globe2 :size="15" /></template>
                联网
              </BaseButton>
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

            <div class="form-field">
              <label class="form-field__label" for="game-args">启动参数</label>
              <TextField id="game-args" v-model="form.args" placeholder="可选，例如 -windowed" />
            </div>
          </div>
        </div>

        <p v-if="displayedError" class="form-error">{{ displayedError }}</p>
      </form>
    </div>

    <template #footer>
      <BaseButton variant="secondary" type="button" :disabled="saving" @click="emit('close')">取消</BaseButton>
      <BaseButton variant="primary" type="button" :loading="saving" @click="submitForm">{{ submitText }}</BaseButton>
    </template>
  </BaseModal>

  <OnlineCoverDialog
    :open="isOnlineCoverDialogOpen"
    :initial-query="form.name"
    @close="isOnlineCoverDialogOpen = false"
    @select="selectRemoteCover"
  />
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
    gap: 12px;
    min-width: 0;
  }

  .cover-preview {
    display: grid;
    align-content: start;
    gap: 8px;
  }

  .cover-preview__actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
    width: 132px;
  }

  .cover-preview__action:only-child {
    grid-column: 1 / -1;
  }

  .cover-preview__action {
    gap: 4px;
    min-width: 0;
    padding-inline: 6px;
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
    background: var(--surface);
  }

  .form-field {
    display: grid;
    gap: 7px;
    min-width: 0;
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

    .cover-preview__actions {
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

  @media (max-width: 560px) {
    .game-form__hero {
      grid-template-columns: 1fr;
    }

    .cover-preview {
      justify-items: center;
    }
  }
</style>
