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
    <span v-if="loading" class="base-button__spinner" aria-hidden="true" />
    <span v-else-if="$slots.icon" class="base-button__icon">
      <slot name="icon" />
    </span>
    <span class="base-button__label"><slot /></span>
  </button>
</template>

<style scoped>
  .base-button {
    font-size: var(--font-size-md);
    display: inline-flex;
    gap: 8px;
    align-items: center;
    justify-content: center;
    border: 1px solid transparent;
    border-radius: 8px;
    color: var(--text);
    white-space: nowrap;
    transition:
      border-color 160ms ease,
      background 160ms ease,
      color 160ms ease,
      opacity 160ms ease;
  }

  .base-button--md {
    min-height: 34px;
    padding: 0 12px;
  }

  .base-button--sm {
    min-height: 30px;
    padding: 0 10px;
    font-size: 13px;
  }

  .base-button:active:not(:disabled) {
    background: var(--surface-hover);
  }

  .base-button:disabled {
    opacity: 0.52;
  }

  .base-button__spinner {
    width: 14px;
    height: 14px;
    border: 2px solid currentColor;
    border-right-color: transparent;
    border-radius: 999px;
    animation: button-spin 680ms linear infinite;
  }

  .base-button--primary {
    background: linear-gradient(180deg, #8d73ff, #6d50e8);
    color: #ffffff;
    font-weight: 700;
    box-shadow: 0 12px 28px rgba(41, 30, 110, 0.34);
  }

  .base-button--secondary {
    border-color: var(--border);
    background: var(--surface);
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
    background: var(--surface-hover);
    color: var(--text);
  }

  .base-button--danger {
    border-color: rgba(248, 113, 113, 0.28);
    background: linear-gradient(180deg, #fb5576, #e3375e);
    color: #ffffff;
    font-weight: 700;
  }

  .base-button__icon,
  .base-button__label {
    display: inline-flex;
    align-items: center;
  }

  @keyframes button-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
