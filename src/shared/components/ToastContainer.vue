<script setup lang="ts">
  import { AlertCircle, CheckCircle2, Info } from '@lucide/vue'
  import { computed } from 'vue'
  import { useToast, type ToastMessage, type ToastType } from '../composables/useToast'

  const toast = useToast()

  const iconByType: Record<ToastType, unknown> = {
    success: CheckCircle2,
    error: AlertCircle,
    info: Info
  }

  const toastItems = computed(() => toast.toasts.value)

  function getMessage(item: ToastMessage) {
    return item.description ? `${item.title}：${item.description}` : item.title
  }
</script>

<template>
  <Teleport to="body">
    <TransitionGroup name="toast" tag="section" class="toast-stack" aria-live="polite" aria-label="操作反馈">
      <article
        v-for="item in toastItems"
        :key="item.id"
        class="toast-message"
        :class="[`toast-message--${item.type}`]"
        role="status"
      >
        <component :is="iconByType[item.type]" class="toast-message__icon" :size="17" aria-hidden="true" />
        <span class="toast-message__text">{{ getMessage(item) }}</span>
      </article>
    </TransitionGroup>
  </Teleport>
</template>

<style scoped>
  .toast-stack {
    position: fixed;
    z-index: 140;
    top: 20px;
    left: 50%;
    display: grid;
    width: min(520px, calc(100vw - 32px));
    gap: 10px;
    justify-items: center;
    pointer-events: none;
    transform: translateX(-50%);
  }

  .toast-message {
    display: inline-grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    align-items: center;
    max-width: 100%;
    min-height: 40px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: rgba(31, 28, 37, 0.96);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.34);
    color: var(--text);
    padding: 9px 14px;
    pointer-events: auto;
    backdrop-filter: blur(18px);
  }

  .toast-message__icon {
    color: var(--accent-strong);
  }

  .toast-message--success .toast-message__icon {
    color: #86efac;
  }

  .toast-message--error .toast-message__icon {
    color: #fecdd3;
  }

  .toast-message__text {
    overflow: hidden;
    font-size: var(--font-size-md);
    font-weight: 600;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toast-enter-active,
  .toast-leave-active {
    transition:
      opacity 180ms ease,
      transform 180ms ease;
  }

  .toast-enter-from,
  .toast-leave-to {
    opacity: 0;
    transform: translateY(-8px);
  }

  .toast-move {
    transition: transform 180ms ease;
  }

  @media (max-width: 720px) {
    .toast-stack {
      top: 12px;
      width: calc(100vw - 24px);
    }

    .toast-message {
      width: 100%;
    }
  }
</style>
