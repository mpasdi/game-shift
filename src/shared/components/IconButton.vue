<script setup lang="ts">
  type IconButtonVariant = 'plain' | 'active' | 'danger'

  withDefaults(
    defineProps<{
      label: string
      variant?: IconButtonVariant
      disabled?: boolean
      type?: 'button' | 'submit' | 'reset'
    }>(),
    {
      variant: 'plain',
      disabled: false,
      type: 'button'
    }
  )

  const emit = defineEmits<{
    click: [event: MouseEvent]
  }>()
</script>

<template>
  <button
    class="icon-button"
    :class="[`icon-button--${variant}`]"
    :type="type"
    :disabled="disabled"
    :title="label"
    :aria-label="label"
    @click="emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped>
  .icon-button {
    display: grid;
    width: 38px;
    min-width: 38px;
    height: 36px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    line-height: 0;
    padding: 0;
    place-items: center;
    transition:
      background 160ms ease,
      border-color 160ms ease,
      color 160ms ease;
  }

  .icon-button:hover {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
  }

  .icon-button:active {
    background: var(--surface-hover);
  }

  .icon-button:disabled {
    cursor: not-allowed;
    opacity: 0.48;
  }

  .icon-button:disabled:hover {
    border-color: transparent;
    background: transparent;
    color: var(--text-muted);
  }

  .icon-button--active {
    border-color: var(--accent-border);
    background: var(--accent-soft);
    color: var(--accent-strong);
  }

  .icon-button--active:disabled {
    border-color: var(--accent-border);
    background: var(--accent-soft);
    color: var(--accent-strong);
  }

  .icon-button--danger:hover {
    background: var(--danger-soft);
    color: #fecdd3;
  }

  .icon-button :deep(svg) {
    display: block;
  }
</style>
