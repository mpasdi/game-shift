<script setup lang="ts">
  import { X } from '@lucide/vue'
  import { DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from 'reka-ui'

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

  function handleOpenChange(open: boolean) {
    if (!open) requestClose()
  }

  function handleInteractOutside(event: Event) {
    if (!props.closeOnBackdrop || props.closeDisabled) event.preventDefault()
  }

  function handleEscapeKeyDown(event: KeyboardEvent) {
    if (props.closeDisabled) event.preventDefault()
  }
</script>

<template>
  <DialogRoot :open="props.open" @update:open="handleOpenChange">
    <DialogPortal>
      <DialogOverlay class="modal-backdrop" />
      <DialogContent
        class="modal-panel"
        :class="`modal-panel--${size}`"
        :aria-describedby="undefined"
        @interact-outside="handleInteractOutside"
        @escape-key-down="handleEscapeKeyDown"
      >
        <header class="modal-header">
          <DialogTitle as-child>
            <h2>{{ title }}</h2>
          </DialogTitle>
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
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<!-- DialogPortal 挂载到 body，弹框样式需保持非 scoped。 -->
<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(8, 7, 10, 0.78);
    backdrop-filter: blur(14px);
    animation: fade-in 150ms ease-out both;
  }

  .modal-panel {
    position: fixed;
    z-index: 101;
    top: 50%;
    left: 50%;
    display: flex;
    width: min(560px, calc(100vw - 44px));
    flex-direction: column;
    max-height: min(760px, calc(100vh - 44px));
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    outline: 0;
    background: var(--panel-strong);
    box-shadow: var(--shadow);
    animation: modal-in 180ms ease-out both;
  }

  .modal-panel--sm {
    width: min(420px, calc(100vw - 44px));
  }

  .modal-panel--sm .modal-footer {
    justify-content: center;
  }

  .modal-panel--lg {
    width: min(760px, calc(100vw - 44px));
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

  .modal-close:focus-visible {
    outline: 0;
    box-shadow: var(--control-focus-shadow);
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
      transform: translate(-50%, calc(-50% + 12px)) scale(0.98);
    }

    to {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
    }
  }

  @media (max-width: 720px) {
    .modal-footer {
      align-items: stretch;
      flex-direction: column;
    }

    .modal-footer .base-button {
      width: 100%;
    }
  }
</style>
