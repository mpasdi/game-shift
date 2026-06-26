<script setup lang="ts">
  type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
  type ButtonSize = 'sm' | 'md'

  withDefaults(
    defineProps<{
      variant?: ButtonVariant
      size?: ButtonSize
      disabled?: boolean
      loading?: boolean
      type?: 'button' | 'submit' | 'reset'
    }>(),
    {
      variant: 'secondary',
      size: 'md',
      disabled: false,
      loading: false,
      type: 'button'
    }
  )
</script>

<template>
  <button
    class="base-button"
    :class="[`base-button--${variant}`, `base-button--${size}`]"
    :type="type"
    :disabled="disabled || loading"
  >
    <span v-if="$slots.icon" class="base-button__icon">
      <slot name="icon" />
    </span>
    <span class="base-button__label"><slot /></span>
  </button>
</template>

<style scoped>
  .base-button {
    display: inline-flex;
    gap: 8px;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: 8px;
    white-space: nowrap;
    transition:
      transform 160ms ease,
      border-color 160ms ease,
      background 160ms ease,
      color 160ms ease,
      opacity 160ms ease;
  }

  .base-button--md {
    min-height: 38px;
    padding: 0 14px;
  }

  .base-button--sm {
    min-height: 32px;
    padding: 0 11px;
    font-size: 13px;
  }

  .base-button:hover:not(:disabled) {
    transform: translateY(-1px);
  }

  .base-button:active:not(:disabled) {
    transform: translateY(0) scale(0.98);
  }

  .base-button:disabled {
    opacity: 0.52;
  }

  .base-button--primary {
    background: linear-gradient(135deg, #a78bfa, #7c3aed);
    color: #ffffff;
    font-weight: 760;
    box-shadow: 0 14px 34px rgba(124, 58, 237, 0.3);
  }

  .base-button--secondary {
    border-color: var(--border);
    background: rgba(255, 255, 255, 0.07);
    color: var(--text);
  }

  .base-button--ghost {
    border-color: transparent;
    background: transparent;
    color: var(--text-muted);
  }

  .base-button--ghost:hover:not(:disabled),
  .base-button--secondary:hover:not(:disabled) {
    border-color: var(--border-strong);
    background: rgba(255, 255, 255, 0.11);
    color: var(--text);
  }

  .base-button--danger {
    background: var(--danger-soft);
    color: #fecdd3;
  }

  .base-button__icon,
  .base-button__label {
    display: inline-flex;
    align-items: center;
  }
</style>
