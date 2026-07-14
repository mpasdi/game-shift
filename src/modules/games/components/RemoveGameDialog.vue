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
  <BaseModal :open="props.open" title="移除游戏" size="sm" @close="emit('close')">
    <div class="confirm-dialog">
      <div class="confirm-dialog__icon" aria-hidden="true">
        <Trash2 :size="30" />
      </div>
      <div class="confirm-dialog__content">
        <!-- prettier-ignore -->
        <p v-if="props.game" class="confirm-dialog__title">确定移除<span class="confirm-dialog__game-name">「{{ props.game.name }}」</span>吗？</p>
        <p v-else class="confirm-dialog__title">确定要移除这个游戏吗？</p>
        <template v-if="props.game">
          <div class="confirm-dialog__target">
            <span class="confirm-dialog__path" :title="props.game.exePath">{{ props.game.exePath }}</span>
          </div>
        </template>
        <p class="confirm-dialog__description">只会移除 Game Shift 中的记录，不会删除本地磁盘上的游戏文件。</p>
        <p v-if="props.errorMessage" class="form-error">{{ props.errorMessage }}</p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" type="button" :disabled="props.deleting" @click="emit('close')">取消</BaseButton>
      <BaseButton variant="danger" type="button" :loading="props.deleting" @click="emit('confirm')">移除</BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .confirm-dialog {
    display: grid;
    gap: 16px;
    justify-items: center;
    padding: 8px 6px 4px;
    text-align: center;
  }

  .confirm-dialog__icon {
    display: grid;
    width: 68px;
    height: 68px;
    place-items: center;
    border: 1px solid rgba(248, 113, 113, 0.45);
    border-radius: 999px;
    background: var(--danger-soft);
    color: var(--danger);
  }

  .confirm-dialog__content {
    min-width: 0;
  }

  .confirm-dialog__title {
    margin: 0;
    color: var(--text);
    font-weight: 700;
  }

  .confirm-dialog__game-name {
    color: var(--danger);
  }

  .confirm-dialog__target {
    display: grid;
    max-width: 340px;
    margin: 7px auto 0;
  }

  .confirm-dialog__path {
    overflow-wrap: anywhere;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    line-height: 1.35;
  }

  .confirm-dialog__description {
    color: var(--text-subtle);
    font-size: var(--font-size-sm);
    line-height: 1.5;
  }

  .confirm-dialog__description {
    margin: 8px 0 0;
  }

  .form-error {
    margin: 0;
    color: #fecdd3;
    font-size: var(--font-size-md);
  }
</style>
