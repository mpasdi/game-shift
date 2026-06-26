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
    border: 0;
    background: transparent;
    color: var(--text-muted);
    place-items: center;
    transition:
      background 160ms ease,
      color 160ms ease,
      transform 160ms ease;
  }

  .icon-button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }

  .icon-button:active {
    transform: scale(0.94);
  }

  .icon-button--active {
    background: var(--accent-soft);
    color: #ddd6fe;
  }

  .icon-button--danger:hover {
    background: var(--danger-soft);
    color: #fecdd3;
  }
</style>
