<script setup lang="ts">
  type IconButtonVariant = 'plain' | 'active'

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
