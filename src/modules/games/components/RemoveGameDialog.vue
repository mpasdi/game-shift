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
