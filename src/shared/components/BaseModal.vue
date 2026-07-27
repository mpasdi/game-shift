<script setup lang="ts">
  import { X } from '@lucide/vue'
  const props = withDefaults(
    defineProps<{
      open: boolean
      title: string
      size?: 'sm' | 'md' | 'lg'
      bodyScrollable?: boolean
      closeOnBackdrop?: boolean
      closeDisabled?: boolean
    }>(),
    {
      size: 'md',
      bodyScrollable: true,
      closeOnBackdrop: true,
      closeDisabled: false
    }
  )

  const emit = defineEmits<{
    close: []
  }>()

  function requestClose() {
    if (props.closeDisabled) return
    emit('close')
  }

  function closeFromBackdrop() {
    if (!props.closeOnBackdrop) return
    requestClose()
  }
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation" @click.self="closeFromBackdrop">
      <section class="modal-panel" :class="`modal-panel--${size}`" role="dialog" aria-modal="true" :aria-label="title">
        <header class="modal-header">
          <h2>{{ title }}</h2>
          <button class="modal-close" type="button" aria-label="关闭" :disabled="closeDisabled" @click="requestClose">
            <X :size="16" :stroke-width="2.4" />
          </button>
        </header>
        <div class="modal-body" :class="{ 'modal-body--fixed': !bodyScrollable }">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="modal-footer">
          <slot name="footer" />
        </footer>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: rgba(8, 7, 10, 0.78);
    padding: 22px;
    backdrop-filter: blur(14px);
    animation: fade-in 150ms ease-out both;
  }

  .modal-panel {
    display: flex;
    width: min(560px, 100%);
    flex-direction: column;
    max-height: min(760px, calc(100vh - 44px));
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: var(--panel-strong);
    box-shadow: var(--shadow);
    animation: modal-in 180ms ease-out both;
  }

  .modal-panel--sm {
    width: min(420px, 100%);
  }

  .modal-panel--sm .modal-footer {
    justify-content: center;
  }

  .modal-panel--lg {
    width: min(760px, 100%);
  }

  .modal-header,
  .modal-footer {
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: flex-end;
    padding: 8px 24px;
  }

  .modal-header {
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h2 {
    margin: 0;
    color: var(--text);
    font-size: 15px;
    font-weight: 700;
    line-height: 1.25;
  }

  .modal-close {
    display: grid;
    width: 30px;
    height: 30px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    place-items: center;
    line-height: 0;
  }

  .modal-close:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text);
  }

  .modal-close:disabled {
    cursor: not-allowed;
    opacity: 0.42;
  }

  .modal-body {
    min-height: 0;
    overflow: auto;
    padding: 24px;
  }

  .modal-body--fixed {
    overflow: visible;
  }

  .modal-footer {
    border-top: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.025);
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }

    to {
      opacity: 1;
    }
  }

  @keyframes modal-in {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.98);
    }

    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (max-width: 720px) {
    .modal-footer {
      align-items: stretch;
      flex-direction: column;
    }

    .modal-footer :deep(.base-button) {
      width: 100%;
    }
  }
</style>
