<script setup lang="ts">
  import { Trash2 } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import type { Game } from '../types/game'

  const props = defineProps<{
    open: boolean
    game?: Game | null
    deleting: boolean
    errorMessage?: string | null
  }>()

  const emit = defineEmits<{
    close: []
    confirm: []
  }>()
</script>

<template>
  <BaseModal :open="props.open" title="移除游戏" @close="emit('close')">
    <div class="confirm-dialog">
      <div class="confirm-dialog__icon" aria-hidden="true">
        <Trash2 :size="22" />
      </div>
      <div class="confirm-dialog__content">
        <p class="confirm-dialog__title">确认从游戏库移除 {{ props.game?.name ?? '该游戏' }}？</p>
        <p class="confirm-dialog__description">只会删除 Game Shift 中的记录，不会删除本地磁盘上的游戏文件。</p>
        <p v-if="props.game" class="confirm-dialog__path" :title="props.game.exePath">{{ props.game.exePath }}</p>
        <p v-if="props.errorMessage" class="form-error">{{ props.errorMessage }}</p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" type="button" :disabled="props.deleting" @click="emit('close')">取消</BaseButton>
      <BaseButton variant="danger" type="button" :loading="props.deleting" @click="emit('confirm')">
        确认移除
      </BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .confirm-dialog {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }

  .confirm-dialog__icon {
    display: grid;
    width: 42px;
    height: 42px;
    place-items: center;
    border-radius: 8px;
    background: var(--danger-soft);
    color: #fecdd3;
  }

  .confirm-dialog__content {
    min-width: 0;
  }

  .confirm-dialog__title {
    margin: 0;
    color: var(--text);
    font-weight: 760;
  }

  .confirm-dialog__description,
  .confirm-dialog__path {
    color: var(--text-muted);
    line-height: 1.6;
  }

  .confirm-dialog__description {
    margin: 8px 0 0;
  }

  .confirm-dialog__path {
    overflow-wrap: anywhere;
    margin: 10px 0 0;
    font-size: 13px;
  }

  .form-error {
    margin: 0;
    color: #fecdd3;
    font-size: 13px;
  }
</style>
