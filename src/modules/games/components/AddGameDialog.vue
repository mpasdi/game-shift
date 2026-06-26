<script setup lang="ts">
  import { computed, reactive, ref, watch } from 'vue'
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
  <BaseModal :open="props.open" :title="modalTitle" @close="emit('close')">
    <form class="game-form" @submit.prevent="submitForm">
      <div class="path-field">
        <TextField id="game-exe" v-model="form.exePath" label="启动程序" placeholder="选择游戏 .exe 文件" readonly />
        <BaseButton variant="secondary" type="button" @click="chooseExePath">
          <template #icon><FileSearch :size="17" /></template>
          选择
        </BaseButton>
      </div>

      <TextField id="game-name" v-model="form.name" label="游戏名称" placeholder="例如：Elden Ring" />

      <div class="path-field">
        <TextField
          id="game-work-dir"
          v-model="form.workDir"
          label="工作目录"
          placeholder="默认使用 .exe 所在目录"
          readonly
        />
        <BaseButton variant="secondary" type="button" @click="chooseWorkDir">
          <template #icon><FolderOpen :size="17" /></template>
          选择
        </BaseButton>
      </div>

      <TextField id="game-args" v-model="form.args" label="启动参数" placeholder="可选，例如 -windowed" />

      <p v-if="displayedError" class="form-error">{{ displayedError }}</p>
    </form>

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

  .path-field {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
  }

  .form-error {
    margin: 0;
    color: #fecdd3;
    font-size: 13px;
  }

  @media (max-width: 720px) {
    .path-field {
      align-items: stretch;
      grid-template-columns: 1fr;
    }

    .path-field :deep(.base-button) {
      width: 100%;
    }
  }
</style>
