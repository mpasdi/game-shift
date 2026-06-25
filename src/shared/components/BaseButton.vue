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
