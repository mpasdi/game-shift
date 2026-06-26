<script setup lang="ts">
  defineProps<{
    open: boolean
    title: string
  }>()

  const emit = defineEmits<{
    close: []
  }>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation" @click.self="emit('close')">
      <section class="modal-panel" role="dialog" aria-modal="true" :aria-label="title">
        <header class="modal-header">
          <h2>{{ title }}</h2>
          <button class="modal-close" type="button" aria-label="关闭" @click="emit('close')">×</button>
        </header>
        <div class="modal-body">
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
    background: rgba(9, 7, 13, 0.74);
    padding: 22px;
    backdrop-filter: blur(14px);
    animation: fade-in 150ms ease-out both;
  }

  .modal-panel {
    width: min(560px, 100%);
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: rgba(31, 27, 41, 0.98);
    box-shadow: var(--shadow);
    animation: modal-in 180ms ease-out both;
  }

  .modal-header,
  .modal-footer {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px;
  }

  .modal-header {
    border-bottom: 1px solid var(--border);
  }

  .modal-header h2 {
    margin: 0;
    color: var(--text);
    font-size: 18px;
    line-height: 1.25;
  }

  .modal-close {
    display: grid;
    width: 34px;
    height: 34px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    place-items: center;
    font-size: 22px;
    line-height: 1;
  }

  .modal-close:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }

  .modal-body {
    padding: 20px;
  }

  .modal-footer {
    border-top: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.035);
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
